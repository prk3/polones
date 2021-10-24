import cv2
import sys

img = cv2.imread(sys.argv[1])
assert(img.shape[0] % 8 == 0)
assert(img.shape[1] % 8 == 0)
bytes = []
for y in range(img.shape[0]):
    for x in range(0, img.shape[1], 8):
        byte = 0
        for bit in range(8):
            set_ = 1
            if img[y][x+bit][0] == 0 and img[y][x+bit][1] == 0 and img[y][x+bit][2] == 0:
                set_ = 0
            byte = (byte << 1) | set_
        bytes.append(byte)

out = open(sys.argv[2], 'w+b')
out_content = bytearray(bytes)
out.write(out_content)
out.close()
