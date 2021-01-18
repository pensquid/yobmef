# Used in the python repl to play with bitboards.
# The implementation is horrible and lacks many desirable features.

class BB:
    def __init__(self, n=0):
        self.n = n


    @property
    def bin(self):
        return ''.join(
            f'{self.get(n)} ' if (n+1) % 8 == 0 else f'{self.get(n)}'
            for n in range(64)
        ).strip()

    def get(self, n):
        return int(self.n & (1 << n) != 0)

    def set(self, n):
        self.n = self.n | (1 << n)

    def __str__(self):
        return '\n'.join(reversed([
            ' '.join([str(self.get(i*8 + j)) for j in range(8)])
            for i in range(8)
        ]))

    def __repr__(self):
        return str(self)

b = BB(0)
b.set(0)
b.set(1)
b.set(8)
print(b)
