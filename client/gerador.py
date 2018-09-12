with open('test.txt', 'a') as f:
    for i in range(10000):
        f.write('testmessage\n')
