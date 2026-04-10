#!/usr/bin/env python3
import argparse
import asyncio
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from bleak import BleakScanner
from bleak.exc import BleakError

REPO_ROOT = Path(__file__).resolve().parents[3]
DEFAULT_TLSRPGM = REPO_ROOT / "TlsrPgm.py"
DEFAULT_NM = REPO_ROOT / "tc32-vendor/bin/tc32-elf-nm"
DEFAULT_ELF = REPO_ROOT / "tlsr82xx/target/tc32-unknown-none-elf/release/tlsr82xx-ble-ota-rust8258"


@dataclass
class Status:
    magic: int
    version: int
    boot_count: int
    loop_count: int
    phase: int
    last_error: int
    event_ok: int
    event_fail: int
    tx_ok: int
    tx_timeout: int
    conn_events: int
    disconn_events: int
    link_activity_events: int
    link_state: int
    last_rx_pdu_type: int
    last_rx_pdu_len: int
    last_rx_target_match: int
    last_rx_init_addr0: int
    last_rx_init_addr1: int
    last_conn_aa: int
    last_conn_interval: int
    last_conn_timeout: int
    last_conn_hop: int
    conn_listen_armed: int
    conn_data_channel: int
    conn_data_rx_count: int
    last_conn_data_llid: int
    last_conn_data_len: int
    conn_ll_ctrl_rx_count: int
    last_conn_ll_ctrl_opcode: int
    conn_att_rx_count: int
    last_conn_att_opcode: int
    conn_att_rsp_count: int
    conn_att_tx_attempt_count: int
    conn_att_tx_ok_count: int
    last_conn_att_rsp_opcode: int


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Smoke-check TLSR8258 BLE OTA advert + RAM status")
    p.add_argument("--port", default="/dev/cu.usbserial-10", help="programmer serial port")
    p.add_argument("--timeout", type=float, default=8.0, help="BLE scan duration in seconds")
    p.add_argument("--name", default="TLS", help="name substring filter")
    p.add_argument(
        "--service",
        default="1912",
        help="service UUID substring filter (example: 1912)",
    )
    p.add_argument(
        "--status-addr",
        default="",
        help="BLE_BEACON_STATUS symbol address (auto-resolve if omitted)",
    )
    p.add_argument(
        "--status-size",
        default="0x90",
        help="status dump size in bytes",
    )
    p.add_argument(
        "--skip-scan",
        action="store_true",
        help="skip BLE scan and validate only RAM telemetry",
    )
    p.add_argument(
        "--require-scan",
        action="store_true",
        help="fail if BLE scan is unavailable or no matching advert found",
    )
    p.add_argument(
        "--ds-retries",
        type=int,
        default=3,
        help="number of retries for ds reads",
    )
    p.add_argument(
        "--elf",
        default=str(DEFAULT_ELF),
        help="ELF path for auto-resolving BLE_BEACON_STATUS",
    )
    p.add_argument(
        "--nm",
        default=str(DEFAULT_NM),
        help="nm tool path used for symbol lookup",
    )
    p.add_argument(
        "--tlsrpgm",
        default=str(DEFAULT_TLSRPGM),
        help="path to TlsrPgm.py",
    )
    return p.parse_args()


def le_u32(buf: bytes, off: int) -> int:
    return int.from_bytes(buf[off : off + 4], "little")


def parse_status(buf: bytes) -> Status:
    return Status(
        magic=le_u32(buf, 0),
        version=le_u32(buf, 4),
        boot_count=le_u32(buf, 8),
        loop_count=le_u32(buf, 12),
        phase=buf[16],
        last_error=buf[17],
        event_ok=le_u32(buf, 20),
        event_fail=le_u32(buf, 24),
        tx_ok=le_u32(buf, 32),
        tx_timeout=le_u32(buf, 36),
        conn_events=le_u32(buf, 44),
        disconn_events=le_u32(buf, 48),
        link_activity_events=le_u32(buf, 52),
        link_state=buf[56],
        last_rx_pdu_type=buf[57],
        last_rx_pdu_len=buf[58],
        last_rx_target_match=buf[59],
        last_rx_init_addr0=buf[61],
        last_rx_init_addr1=buf[62],
        last_conn_aa=le_u32(buf, 64),
        last_conn_interval=int.from_bytes(buf[68:70], "little"),
        last_conn_timeout=int.from_bytes(buf[70:72], "little"),
        last_conn_hop=buf[72],
        conn_listen_armed=buf[73],
        conn_data_channel=buf[74],
        conn_data_rx_count=le_u32(buf, 104),
        last_conn_data_llid=buf[108],
        last_conn_data_len=buf[109],
        conn_ll_ctrl_rx_count=le_u32(buf, 112),
        last_conn_ll_ctrl_opcode=buf[116],
        conn_att_rx_count=le_u32(buf, 120),
        last_conn_att_opcode=buf[124],
        conn_att_rsp_count=le_u32(buf, 128),
        conn_att_tx_attempt_count=le_u32(buf, 132),
        conn_att_tx_ok_count=le_u32(buf, 136),
        last_conn_att_rsp_opcode=buf[140],
    )


def parse_ds_output(text: str) -> bytes:
    out = bytearray()
    for line in text.splitlines():
        m = re.match(r"^[0-9a-fA-F]{6,8}:\s+(.+)$", line.strip())
        if not m:
            continue
        for tok in m.group(1).split():
            if re.fullmatch(r"[0-9a-fA-F]{2}", tok):
                out.append(int(tok, 16))
    return bytes(out)


def resolve_status_addr(args: argparse.Namespace) -> str:
    if args.status_addr:
        return args.status_addr

    elf = Path(args.elf)
    if not elf.exists():
        raise RuntimeError(f"ELF not found: {elf}")
    proc = subprocess.run([args.nm, "-n", str(elf)], capture_output=True, text=True)
    if proc.returncode != 0:
        raise RuntimeError(f"nm failed with code {proc.returncode}")
    for line in proc.stdout.splitlines():
        if " BLE_BEACON_STATUS" in line:
            return "0x" + line.split()[0]
    raise RuntimeError("BLE_BEACON_STATUS symbol not found")


def read_status_once(args: argparse.Namespace) -> Status:
    status_addr = resolve_status_addr(args)
    cmd = [
        "python3",
        args.tlsrpgm,
        "-p",
        args.port,
        "-t",
        "50",
        "-a",
        "100",
        "-g",
        "ds",
        status_addr,
        args.status_size,
    ]
    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        print(proc.stdout)
        print(proc.stderr, file=sys.stderr)
        raise RuntimeError(f"ds failed with code {proc.returncode}")
    raw = parse_ds_output(proc.stdout)
    need = int(args.status_size, 0)
    if len(raw) < need:
        raise RuntimeError(f"short ds dump: got {len(raw)} bytes, need {need}")
    return parse_status(raw)


def read_status(args: argparse.Namespace) -> Status:
    retries = args.ds_retries if args.ds_retries > 0 else 1
    last_error: Exception | None = None
    for _ in range(retries):
        try:
            return read_status_once(args)
        except Exception as e:
            last_error = e
            continue
    raise RuntimeError(f"ds failed after {retries} attempts: {last_error}")


async def scan(args: argparse.Namespace) -> bool:
    devices = await BleakScanner.discover(timeout=args.timeout, return_adv=True)
    found = False
    for dev, adv in devices.values():
        name = adv.local_name or dev.name or ""
        uuids = list(adv.service_uuids or [])
        if args.name and args.name.lower() not in name.lower():
            continue
        if args.service and not any(args.service.lower() in u.lower() for u in uuids):
            continue
        print(
            f"adv: addr={dev.address} name={name!r} rssi={getattr(adv, 'rssi', None)} "
            f"services={uuids}"
        )
        found = True
    return found


async def main() -> int:
    args = parse_args()
    try:
        status_addr = resolve_status_addr(args)
        print(f"status_addr={status_addr}")
    except Exception as e:
        print(f"status_addr_error={e}")
        print("pass=False")
        return 1

    adv_found = False
    scan_ok = False
    scan_available = True
    if args.skip_scan:
        print("== BLE scan ==")
        print("scan skipped by --skip-scan")
    else:
        print("== BLE scan ==")
        try:
            adv_found = await scan(args)
            scan_ok = True
            print(f"advert_found={adv_found}")
        except BleakError as e:
            scan_available = False
            print(f"scan_error={e}")

    print("== RAM status ==")
    try:
        st = read_status(args)
    except Exception as e:
        print(f"status_error={e}")
        print("pass=False")
        return 1
    print(
        "status:"
        f" magic=0x{st.magic:08x}"
        f" ver={st.version}"
        f" boot={st.boot_count}"
        f" loop={st.loop_count}"
        f" phase={st.phase}"
        f" err={st.last_error}"
        f" ev_ok={st.event_ok}"
        f" ev_fail={st.event_fail}"
        f" tx_ok={st.tx_ok}"
        f" tx_to={st.tx_timeout}"
        f" conn={st.conn_events}"
        f" disconn={st.disconn_events}"
        f" link_act={st.link_activity_events}"
        f" link_state={st.link_state}"
        f" last_rx_pdu=0x{st.last_rx_pdu_type:02x}"
        f" last_rx_len={st.last_rx_pdu_len}"
        f" last_rx_match={st.last_rx_target_match}"
        f" last_rx_init={st.last_rx_init_addr1:02x}:{st.last_rx_init_addr0:02x}"
        f" conn_aa=0x{st.last_conn_aa:08x}"
        f" conn_itvl={st.last_conn_interval}"
        f" conn_to={st.last_conn_timeout}"
        f" conn_hop={st.last_conn_hop}"
        f" conn_listen={st.conn_listen_armed}"
        f" conn_ch={st.conn_data_channel}"
        f" data_rx={st.conn_data_rx_count}"
        f" data_llid={st.last_conn_data_llid}"
        f" data_len={st.last_conn_data_len}"
        f" ll_ctrl_rx={st.conn_ll_ctrl_rx_count}"
        f" ll_ctrl_op=0x{st.last_conn_ll_ctrl_opcode:02x}"
        f" att_rx={st.conn_att_rx_count}"
        f" att_op=0x{st.last_conn_att_opcode:02x}"
        f" att_rsp={st.conn_att_rsp_count}"
        f" att_tx_try={st.conn_att_tx_attempt_count}"
        f" att_tx_ok={st.conn_att_tx_ok_count}"
        f" att_rsp_op=0x{st.last_conn_att_rsp_opcode:02x}"
    )

    ok_magic = st.magic == 0x4F544152
    ok_alive = st.loop_count > 0 or st.event_ok > 0 or st.tx_ok > 0
    if args.skip_scan:
        scan_pass = True
    elif args.require_scan:
        scan_pass = scan_ok and adv_found
    else:
        # If host BLE is unavailable, do not fail telemetry smoke by default.
        scan_pass = (not scan_available) or (scan_ok and adv_found)
    passed = ok_magic and ok_alive and scan_pass
    print(f"pass={passed}")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(asyncio.run(main()))
