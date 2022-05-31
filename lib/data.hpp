

#ifndef _H_DATA
#define _H_DATA

#include "types.hpp"
#include <vector>

void write_test_curves(std::vector<Bezier2> &curves) {

    curves.resize(19);

	curves[0].e0.x = 1398;
	curves[0].e0.y = 731;
	curves[0].e1.x = 1313;
	curves[0].e1.y = 344;
	curves[0].c.x = 1398;
	curves[0].c.y = 510;
	curves[1].e0.x = 1313;
	curves[1].e0.y = 344;
	curves[1].e1.x = 1071;
	curves[1].e1.y = 89;
	curves[1].c.x = 1229;
	curves[1].c.y = 178;
	curves[2].e0.x = 1071;
	curves[2].e0.y = 89;
	curves[2].e1.x = 698;
	curves[2].e1.y = 0;
	curves[2].c.x = 913;
	curves[2].c.y = 0;
	curves[3].e0.x = 698;
	curves[3].e0.y = 0;
	curves[3].e1.x = 323;
	curves[3].e1.y = 88;
	curves[3].c.x = 481;
	curves[3].c.y = 0;
	curves[4].e0.x = 323;
	curves[4].e0.y = 88;
	curves[4].e1.x = 83;
	curves[4].e1.y = 342;
	curves[4].c.x = 166;
	curves[4].c.y = 176;
	curves[5].e0.x = 83;
	curves[5].e0.y = 342;
	curves[5].e1.x = 0;
	curves[5].e1.y = 731;
	curves[5].c.x = 0;
	curves[5].c.y = 509;
	curves[6].e0.x = 0;
	curves[6].e0.y = 731;
	curves[6].e1.x = 185;
	curves[6].e1.y = 1259;
	curves[6].c.x = 0;
	curves[6].c.y = 1069;
	curves[7].e0.x = 185;
	curves[7].e0.y = 1259;
	curves[7].e1.x = 700;
	curves[7].e1.y = 1450;
	curves[7].c.x = 370;
	curves[7].c.y = 1450;
	curves[8].e0.x = 700;
	curves[8].e0.y = 1450;
	curves[8].e1.x = 1073;
	curves[8].e1.y = 1364;
	curves[8].c.x = 915;
	curves[8].c.y = 1450;
	curves[9].e0.x = 1073;
	curves[9].e0.y = 1364;
	curves[9].e1.x = 1314;
	curves[9].e1.y = 1116;
	curves[9].c.x = 1231;
	curves[9].c.y = 1279;
	curves[10].e0.x = 1314;
	curves[10].e0.y = 1116;
	curves[10].e1.x = 1398;
	curves[10].e1.y = 731;
	curves[10].c.x = 1398;
	curves[10].c.y = 953;
	curves[11].e0.x = 1203;
	curves[11].e0.y = 731;
	curves[11].e1.x = 1071;
	curves[11].e1.y = 1144;
	curves[11].c.x = 1203;
	curves[11].c.y = 994;
	curves[12].e0.x = 1071;
	curves[12].e0.y = 1144;
	curves[12].e1.x = 700;
	curves[12].e1.y = 1294;
	curves[12].c.x = 940;
	curves[12].c.y = 1294;
	curves[13].e0.x = 700;
	curves[13].e0.y = 1294;
	curves[13].e1.x = 326;
	curves[13].e1.y = 1146;
	curves[13].c.x = 458;
	curves[13].c.y = 1294;
	curves[14].e0.x = 326;
	curves[14].e0.y = 1146;
	curves[14].e1.x = 194;
	curves[14].e1.y = 731;
	curves[14].c.x = 194;
	curves[14].c.y = 998;
	curves[15].e0.x = 194;
	curves[15].e0.y = 731;
	curves[15].e1.x = 327;
	curves[15].e1.y = 310;
	curves[15].c.x = 194;
	curves[15].c.y = 466;
	curves[16].e0.x = 327;
	curves[16].e0.y = 310;
	curves[16].e1.x = 698;
	curves[16].e1.y = 155;
	curves[16].c.x = 461;
	curves[16].c.y = 155;
	curves[17].e0.x = 698;
	curves[17].e0.y = 155;
	curves[17].e1.x = 1072;
	curves[17].e1.y = 305;
	curves[17].c.x = 942;
	curves[17].c.y = 155;
	curves[18].e0.x = 1072;
	curves[18].e0.y = 305;
	curves[18].e1.x = 1203;
	curves[18].e1.y = 731;
	curves[18].c.x = 1203;
	curves[18].c.y = 456;
}

#endif // _H_CUBIC2QUAD