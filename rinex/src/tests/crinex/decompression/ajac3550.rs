// 2 epochs from AJAC3550 compressed with RNX2CRX historical tool
use crate::tests::crinex::decompression::run_raw_decompression_test;

const INPUT : &str = "&21 12 21  0  0  0.0000000  0 26G07G08G10G16G18G21G23G26G32R04R05R10R12R19R20R21E04E11E12E19E24E25E31E33S23S36

3&131857102133 3&102745756542 3&25091572300   3&25091565600 3&-411138 3&-320373 3&37350 3&35300              645
3&114374313914 3&89122819839 3&21764705880   3&21764701780 3&2312498 3&1801947 3&50150 3&46300 3&85409382159 3&21764701960 3&1726841 3&52650          847                 8
3&110158976908 3&85838153102 3&20962551380   3&20962547000 3&410954 3&320225 3&53950 3&53850 3&82261561655 3&20962546260 3&306976 3&54950          848                 9
3&112191307034 3&87421772491 3&21349295460   3&21349288380 3&-2230522 3&-1738069 3&51400 3&46850              847
3&126212721076 3&98347599110 3&24017468300   3&24017462340 3&-3054209 3&-2379903 3&43650 3&41600 3&94249805543 3&24017468020 3&-2280603 3&47200          746                 7
3&121982069575 3&95050935890 3&23212414440   3&23212407380 3&2286453 3&1781652 3&45300 3&41950              746
3&114748710661 3&89414573974 3&21835945780   3&21835939420 3&-1881978 3&-1466476 3&49150 3&46250 3&85688981647 3&21835944640 3&-1405380 3&52100          847                 8
3&123656154211 3&96355436093 3&23530980420   3&23530977120 3&-3555861 3&-2770802 3&44150 3&44500 3&92340630316 3&23530977400 3&-2655383 3&46750          747                 7
3&134055065113 3&104458480947 3&25509831680   3&25509828140 3&2836327 3&2210128 3&41600 3&41650 3&100106048706 3&25509827540 3&2118232 3&39550          646                 6
3&119147697073 3&92670417710 3&22249990480 3&22249983480  3&22249983740 3&-1987870 3&-1546121 3&47300 3&43900              7 7
3&111791118357 3&86948637065 3&20912840200 3&20912835040  3&20912834920 3&1741093 3&1354186 3&50600 3&48300              8 8
3&118100743218  3&22155398840    3&-3837262  3&46650               7
3&125288994929 3&97446978579 3&23454362660 3&23454357860  3&23454357960 3&2992452 3&2327463 3&43850 3&39850              7 6
3&129511838195 3&100731412801 3&24210870760 3&24210865680  3&24210864560 3&-2917307 3&-2269021 3&33200 3&35900              5 5
3&108349078730 3&84271501501 3&20261817180 3&20261813760  3&20261812740 3&-1178315 3&-916468 3&40850 3&49450              6 8
3&113210718757 3&88052723060 3&21156125780 3&21156119520  3&21156108580 3&1980596 3&1483972 3&50550 3&47150              8 7
3&150111373216  3&28565244280    3&2782722  3&41550  3&112096154348 3&28565242700 3&2078161 3&40500 3&115020379099 3&28565238840 3&2132230 3&40800 3&113558277190 3&28565240800 3&2105204 3&43550  6                   6       6       7
3&122091776804  3&23233292480    3&1381698  3&48850  3&91172418763 3&23233287620 3&1031745 3&47550 3&93550812899 3&23233284180 3&1058620 3&48250 3&92361615305 3&23233285820 3&1045178 3&50700  8                   7       8       8
3&116725676342  3&22212158080    3&-795834  3&49050  3&87165262313 3&22212153500 3&-594301 3&50100 3&89439123124 3&22212150600 3&-609822 3&51700 3&88302194689 3&22212151820 3&-602054 3&53650  8                   8       8       8
3&139452743474  3&26536974200    3&1939558  3&42600  3&104136787885 3&26536972220 3&1448391 3&40000 3&106853381917 3&26536969000 3&1486176 3&41000 3&105495086875 3&26536970460 3&1467304 3&43500  7                   6       6       7
3&131419383087  3&25008274860    3&-2062297  3&52150  3&98137848730 3&25008272740 3&-1540012 3&50950 3&100697950518 3&25008268940 3&-1580201 3&51950 3&99417906595 3&25008270620 3&-1560102 3&54250  8                   8       8       9
3&133165496647  3&25340551060    3&848869  3&50400  3&99441765540 3&25340550340 3&633888 3&49600 3&102035892853 3&25340546780 3&650424 3&50650 3&100738824168 3&25340548200 3&642164 3&52850  8                   8       8       8
3&145046329913  3&27601394760    3&-3209037  3&42700  3&108313833964 3&27601395200 3&-2396461 3&44150 3&111139398834 3&27601391400 3&-2458818 3&44700 3&109726614871 3&27601393280 3&-2427636 3&47300  7                   7       7       7
3&142291069797  3&27077081020    3&-2190415  3&45950  3&106256338827 3&27077078800 3&-1635710 3&45000 3&109028228029 3&27077075420 3&-1678395 3&46400 3&107642282400 3&27077077120 3&-1657000 3&48550  7                   7       7       8
3&200051837090  3&38068603000    3&2966  3&47900               7
3&197948874430  3&37668418660    3&-1295  3&49100               8
                3

12565090  2390900    -15730  -4400               5
-69270191 -53976759 -13181680   -13181620 -7111 -5542 200 -400 -51727720 -13181720 -5246 0
-12047821 -9387906 -2292660   -2292640 -18979 -14789 -300 -250 -8996724 -2292640 -14287 250
67149668 52324407 12778180   12778260 -15767 -12286 -400 550
91630086 71400065 17436800   17436700 -569 -443 200 -250 68425066 17436800 -587 50
-68490267 -53369033 -13033300   -13033200 -6807 -5305 -900 -1300
56704415 44185261 10790500   10790580 -16370 -12756 300 -300 42344234 10790460 -12325 250
106737879 83172355 20311480   20311740 -4236 -3301 100 -450 79706854 20311400 -3028 100
-85150364 -66350939 -16203500   -16203400 3574 2784 -1300 -1650 -63586267 -16203520 2253 -2150
59986463 46656140 11201900 11202080  11202200 -23544 -18313 -700 -250
-51870071 -40343382 -9703360 -9703320  -9703320 -24497 -19055 -700 -50
115266498  21623660    -9935  -300
-89512095 -69620483 -16756900 -16756720  -16756860 -17868 -13896 -1900 750              6
87523977 68074200 16361480 16361740  16361760 -252 -192 500 200                6
35459621 27579703 6631180 6631140  6631100 -7481 -5818 -500 100
-59330938 -46146275 -11087460 -11087380  -11087340 -5990 -4481 100 -250
-83537883  -15897000    3375  -350  -62382134 -15896580 2336 300 -64009529 -15896400 2731 400 -63195820 -15896700 2480 0
-41373291  -7873120    -5109  -250  -30895614 -7873100 -3844 250 -31701609 -7873080 -3890 -50 -31298610 -7873120 -3892 -300
23943003  4556220    -4626  -250  17879527 4556180 -3463 200 18345935 4556160 -3543 0 18112734 4556220 -3522 -400
-58055564  -11047520    -9140  -200  -43353164 -11047580 -6588 200 -44484130 -11047760 -6878 -50 -43918662 -11047700 -6521 -450
62030604  11804020    -10884  -200  46321559 11804000 -8175 150 47529933 11804160 -8346 50 46925752 11804040 -8293 -350                                      8
-25258953  -4806700    -14014  -300  -18862199 -4806640 -10470 0 -19354265 -4806480 -10717 150 -19108230 -4806620 -10639 -300
96277342  18320860    -419  -750  71895388 18320880 -310 -200 73770910 18321200 -474 250 72833152 18320960 -413 -450  6
65710466  12504280    -12  0  49069493 12504480 21 200 50349554 12504480 -54 -200 49709530 12504240 -122 -600                                      7
-90394  -17080    -84  -100
40482  7380    -115  -150";

const OUTPUT: &str = "
 131857102.133 6 102745756.54245  25091572.300
  25091565.600        -411.138        -320.373          37.350          35.300



 114374313.914 8  89122819.83947  21764705.880
  21764701.780        2312.498        1801.947          50.150          46.300
  85409382.159 8  21764701.960        1726.841          52.650


 110158976.908 8  85838153.10248  20962551.380
  20962547.000         410.954         320.225          53.950          53.850
  82261561.655 9  20962546.260         306.976          54.950


 112191307.034 8  87421772.49147  21349295.460
  21349288.380       -2230.522       -1738.069          51.400          46.850



 126212721.076 7  98347599.11046  24017468.300
  24017462.340       -3054.209       -2379.903          43.650          41.600
  94249805.543 7  24017468.020       -2280.603          47.200


 121982069.575 7  95050935.89046  23212414.440
  23212407.380        2286.453        1781.652          45.300          41.950



 114748710.661 8  89414573.97447  21835945.780
  21835939.420       -1881.978       -1466.476          49.150          46.250
  85688981.647 8  21835944.640       -1405.380          52.100


 123656154.211 7  96355436.09347  23530980.420
  23530977.120       -3555.861       -2770.802          44.150          44.500
  92340630.316 7  23530977.400       -2655.383          46.750


 134055065.113 6 104458480.94746  25509831.680
  25509828.140        2836.327        2210.128          41.600          41.650
 100106048.706 6  25509827.540        2118.232          39.550


 119147697.073 7  92670417.710 7  22249990.480    22249983.480
  22249983.740       -1987.870       -1546.121          47.300          43.900



 111791118.357 8  86948637.065 8  20912840.200    20912835.040
  20912834.920        1741.093        1354.186          50.600          48.300



 118100743.218 7                  22155398.840
                     -3837.262                          46.650



 125288994.929 7  97446978.579 6  23454362.660    23454357.860
  23454357.960        2992.452        2327.463          43.850          39.850



 129511838.195 5 100731412.801 5  24210870.760    24210865.680
  24210864.560       -2917.307       -2269.021          33.200          35.900



 108349078.730 6  84271501.501 8  20261817.180    20261813.760
  20261812.740       -1178.315        -916.468          40.850          49.450



 113210718.757 8  88052723.060 7  21156125.780    21156119.520
  21156108.580        1980.596        1483.972          50.550          47.150



 150111373.216 6                  28565244.280
                      2782.722                          41.550
 112096154.348 6  28565242.700        2078.161          40.500   115020379.099 6
  28565238.840        2132.230          40.800   113558277.190 7  28565240.800
      2105.204          43.550
 122091776.804 8                  23233292.480
                      1381.698                          48.850
  91172418.763 7  23233287.620        1031.745          47.550    93550812.899 8
  23233284.180        1058.620          48.250    92361615.305 8  23233285.820
      1045.178          50.700
 116725676.342 8                  22212158.080
                      -795.834                          49.050
  87165262.313 8  22212153.500        -594.301          50.100    89439123.124 8
  22212150.600        -609.822          51.700    88302194.689 8  22212151.820
      -602.054          53.650
 139452743.474 7                  26536974.200
                      1939.558                          42.600
 104136787.885 6  26536972.220        1448.391          40.000   106853381.917 6
  26536969.000        1486.176          41.000   105495086.875 7  26536970.460
      1467.304          43.500
 131419383.087 8                  25008274.860
                     -2062.297                          52.150
  98137848.730 8  25008272.740       -1540.012          50.950   100697950.518 8
  25008268.940       -1580.201          51.950    99417906.595 9  25008270.620
     -1560.102          54.250
 133165496.647 8                  25340551.060
                       848.869                          50.400
  99441765.540 8  25340550.340         633.888          49.600   102035892.853 8
  25340546.780         650.424          50.650   100738824.168 8  25340548.200
       642.164          52.850
 145046329.913 7                  27601394.760
                     -3209.037                          42.700
 108313833.964 7  27601395.200       -2396.461          44.150   111139398.834 7
  27601391.400       -2458.818          44.700   109726614.871 7  27601393.280
     -2427.636          47.300
 142291069.797 7                  27077081.020
                     -2190.415                          45.950
 106256338.827 7  27077078.800       -1635.710          45.000   109028228.029 7
  27077075.420       -1678.395          46.400   107642282.400 8  27077077.120
     -1657.000          48.550
 200051837.090 7                  38068603.000
                         2.966                          47.900



 197948874.430 8                  37668418.660
                        -1.295                          49.100



 21 12 21  0  0 30.0000000  0 26G07G08G10G16G18G21G23G26G32R04R05R10
                                R12R19R20R21E04E11E12E19E24E25E31E33
                                S23S36
 131869667.223 5                  25093963.200
                      -426.868                          32.950



 114305043.723 8  89068843.08047  21751524.200
  21751520.160        2305.387        1796.405          50.350          45.900
  85357654.439 8  21751520.240        1721.595          52.650


 110146929.087 8  85828765.19648  20960258.720
  20960254.360         391.975         305.436          53.650          53.600
  82252564.931 9  20960253.620         292.689          55.200


 112258456.702 8  87474096.89847  21362073.640
  21362066.640       -2246.289       -1750.355          51.000          47.400



 126304351.162 7  98418999.17546  24034905.100
  24034899.040       -3054.778       -2380.346          43.850          41.350
  94318230.609 7  24034904.820       -2281.190          47.250


 121913579.308 7  94997566.85746  23199381.140
  23199374.180        2279.646        1776.347          44.400          40.650



 114805415.076 8  89458759.23547  21846736.280
  21846730.000       -1898.348       -1479.232          49.450          45.950
  85731325.881 8  21846735.100       -1417.705          52.350


 123762892.090 7  96438608.44847  23551291.900
  23551288.860       -3560.097       -2774.103          44.250          44.050
  92420337.170 7  23551288.800       -2658.411          46.850


 133969914.749 6 104392130.00846  25493628.180
  25493624.740        2839.901        2212.912          40.300          40.000
 100042462.439 6  25493624.020        2120.485          37.400


 119207683.536 7  92717073.850 7  22261192.380    22261185.560
  22261185.940       -2011.414       -1564.434          46.600          43.650



 111739248.286 8  86908293.683 8  20903136.840    20903131.720
  20903131.600        1716.596        1335.131          49.900          48.250



 118216009.716 7                  22177022.500
                     -3847.197                          46.350



 125199482.834 6  97377358.096 6  23437605.760    23437601.140
  23437601.100        2974.584        2313.567          41.950          40.600



 129599362.172 5 100799487.001 6  24227232.240    24227227.420
  24227226.320       -2917.559       -2269.213          33.700          36.100



 108384538.351 6  84299081.204 8  20268448.360    20268444.900
  20268443.840       -1185.796        -922.286          40.350          49.550



 113151387.819 8  88006576.785 7  21145038.320    21145032.140
  21145021.240        1974.606        1479.491          50.650          46.900



 150027835.333 6                  28549347.280
                      2786.097                          41.200
 112033772.214 6  28549346.120        2080.497          40.800   114956369.570 6
  28549342.440        2134.961          41.200   113495081.370 7  28549344.100
      2107.684          43.550
 122050403.513 8                  23225419.360
                      1376.589                          48.600
  91141523.149 7  23225414.520        1027.901          47.800    93519111.290 8
  23225411.100        1054.730          48.200    92330316.695 8  23225412.700
      1041.286          50.400
 116749619.345 8                  22216714.300
                      -800.460                          48.800
  87183141.840 8  22216709.680        -597.764          50.300    89457469.059 8
  22216706.760        -613.365          51.700    88320307.423 8  22216708.040
      -605.576          53.250
 139394687.910 7                  26525926.680
                      1930.418                          42.400
 104093434.721 6  26525924.640        1441.803          40.200   106808897.787 6
  26525921.240        1479.298          40.950   105451168.213 7  26525922.760
      1460.783          43.050
 131481413.691 8                  25020078.880
                     -2073.181                          51.950
  98184170.289 8  25020076.740       -1548.187          51.100   100745480.451 8
  25020073.100       -1588.547          52.000    99464832.347 8  25020074.660
     -1568.395          53.900
 133140237.694 8                  25335744.360
                       834.855                          50.100
  99422903.341 8  25335743.700         623.418          49.600   102016538.588 8
  25335740.300         639.707          50.800   100719715.938 8  25335741.580
       631.525          52.550
 145142607.255 6                  27619715.620
                     -3209.456                          41.950
 108385729.352 7  27619716.080       -2396.771          43.950   111213169.744 7
  27619712.600       -2459.292          44.950   109799448.023 7  27619714.240
     -2428.049          46.850
 142356780.263 7                  27089585.300
                     -2190.427                          45.950
 106305408.320 7  27089583.280       -1635.689          45.200   109078577.583 7
  27089579.900       -1678.449          46.200   107691991.930 7  27089581.360
     -1657.122          47.950
 200051746.696 7                  38068585.920
                         2.882                          47.800



 197948914.912 8                  37668426.040
                        -1.410                          48.950

";

#[test]
fn v1_ajac3550() {
    run_raw_decompression_test(
        true,
        &["GPS", "GLO", "SBAS", "GAL"],
        &[
            "L1", "L2", "C1", "C2", "P1", "P2", "D1", "D2", "S1", "S2", "L5", "C5", "D5", "S5",
            "L7", "C7", "D7", "S7", "L8", "C8", "D8", "S8",
        ],
        INPUT,
        OUTPUT,
    );
}
