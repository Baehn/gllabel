s ='1398 731 1313 344 1398 510 1313 344 1071 89 1229 178 1071 89 698 0 913 0 698 0 323 88 481 0 323 88 83 342 166 176 83 342 0 731 0 509 0 731 185 1259 0 1069 185 1259 700 1450 370 1450 700 1450 1073 1364 915 1450 1073 1364 1314 1116 1231 1279 1314 1116 1398 731 1398 953 1203 731 1071 1144 1203 994 1071 1144 700 1294 940 1294 700 1294 326 1146 458 1294 326 1146 194 731 194 998 194 731 327 310 194 466 327 310 698 155 461 155 698 155 1072 305 942 155 1072 305 1203 731 1203 456'

data_list = s.split(' ')

new_list = [data_list[i:i+6] for i in range(0, len(data_list), 6)]

for i in range(len(new_list)):
    print(f'curves[{i}].e0.x={new_list[i][0]};')
    print(f'curves[{i}].e0.y={new_list[i][1]};')
    print(f'curves[{i}].e1.x={new_list[i][2]};')
    print(f'curves[{i}].e1.y={new_list[i][3]};')
    print(f'curves[{i}].c.x={new_list[i][4]};')
    print(f'curves[{i}].c.y={new_list[i][5]};')