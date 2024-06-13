import re

with open('a.txt', 'r') as file:
    data = file.read().replace('\n', '')
    d = re.sub(r"[\0-z]", "", data)
    print(d)
