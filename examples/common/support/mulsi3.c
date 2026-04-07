unsigned __mulsi3(unsigned a, unsigned b)
{
    unsigned result = 0;

    while (b) {
        if (b & 1u) {
            result += a;
        }
        a <<= 1;
        b >>= 1;
    }

    return result;
}
