import os
from collections import defaultdict

words = defaultdict(lambda: 1)

with os.scandir('./logs/') as dir:
    for path in dir:
        with open(path) as f:
            currentwords = f.read().splitlines()
            for word in currentwords:
                words[word] += 1

print(sum(words.values()))
print(len(words.keys()))

print(max([(word, len(word)) for word in words.keys()], key=lambda x: x[1]))
