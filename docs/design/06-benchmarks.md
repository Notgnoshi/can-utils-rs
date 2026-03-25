# Benchmark results

The [04-candumpr-architecture.md](/docs/design/04-candumpr-architecture.md) design document proposes
three different benchmarks to compare receiver backends.

# Benchmark A - pure CPU cost

```
recv_cost::recv_cost::dedicated run:setup_blocking()
  Instructions:                      454678|N/A                  (*********)
  L1 Hits:                           866202|N/A                  (*********)
  LL Hits:                            10504|N/A                  (*********)
  RAM Hits:                             175|N/A                  (*********)
  Total read+write:                  876881|N/A                  (*********)
  Estimated Cycles:                  924847|N/A                  (*********)
recv_cost::recv_cost::epoll run:setup_nonblocking()
  Instructions:                      519312|N/A                  (*********)
  L1 Hits:                           960184|N/A                  (*********)
  LL Hits:                            10182|N/A                  (*********)
  RAM Hits:                              53|N/A                  (*********)
  Total read+write:                  970419|N/A                  (*********)
  Estimated Cycles:                 1012949|N/A                  (*********)
  Comparison with dedicated run:setup_blocking()
    Instructions:                      454678|519312               (-12.4461%) [-1.14215x]
    L1 Hits:                           866202|960184               (-9.78792%) [-1.10850x]
    LL Hits:                            10504|10182                (+3.16244%) [+1.03162x]
    RAM Hits:                             175|53                   (+230.189%) [+3.30189x]
    Total read+write:                  876881|970419               (-9.63893%) [-1.10667x]
    Estimated Cycles:                  924847|1012949              (-8.69758%) [-1.09526x]
recv_cost::recv_cost::recvmmsg run:setup_nonblocking()
  Instructions:                      468571|N/A                  (*********)
  L1 Hits:                           882905|N/A                  (*********)
  LL Hits:                            10191|N/A                  (*********)
  RAM Hits:                              57|N/A                  (*********)
  Total read+write:                  893153|N/A                  (*********)
  Estimated Cycles:                  935855|N/A                  (*********)
  Comparison with dedicated run:setup_blocking()
    Instructions:                      454678|468571               (-2.96497%) [-1.03056x]
    L1 Hits:                           866202|882905               (-1.89182%) [-1.01928x]
    LL Hits:                            10504|10191                (+3.07134%) [+1.03071x]
    RAM Hits:                             175|57                   (+207.018%) [+3.07018x]
    Total read+write:                  876881|893153               (-1.82186%) [-1.01856x]
    Estimated Cycles:                  924847|935855               (-1.17625%) [-1.01190x]
  Comparison with epoll run:setup_nonblocking()
    Instructions:                      519312|468571               (+10.8289%) [+1.10829x]
    L1 Hits:                           960184|882905               (+8.75281%) [+1.08753x]
    LL Hits:                            10182|10191                (-0.08831%) [-1.00088x]
    RAM Hits:                              53|57                   (-7.01754%) [-1.07547x]
    Total read+write:                  970419|893153               (+8.65093%) [+1.08651x]
    Estimated Cycles:                 1012949|935855               (+8.23781%) [+1.08238x]
recv_cost::recv_cost::uring run:setup_nonblocking()
  Instructions:                      587770|N/A                  (*********)
  L1 Hits:                          1071803|N/A                  (*********)
  LL Hits:                            10210|N/A                  (*********)
  RAM Hits:                             119|N/A                  (*********)
  Total read+write:                 1082132|N/A                  (*********)
  Estimated Cycles:                 1127018|N/A                  (*********)
  Comparison with dedicated run:setup_blocking()
    Instructions:                      454678|587770               (-22.6436%) [-1.29272x]
    L1 Hits:                           866202|1071803              (-19.1827%) [-1.23736x]
    LL Hits:                            10504|10210                (+2.87953%) [+1.02880x]
    RAM Hits:                             175|119                  (+47.0588%) [+1.47059x]
    Total read+write:                  876881|1082132              (-18.9673%) [-1.23407x]
    Estimated Cycles:                  924847|1127018              (-17.9386%) [-1.21860x]
  Comparison with epoll run:setup_nonblocking()
    Instructions:                      519312|587770               (-11.6471%) [-1.13182x]
    L1 Hits:                           960184|1071803              (-10.4141%) [-1.11625x]
    LL Hits:                            10182|10210                (-0.27424%) [-1.00275x]
    RAM Hits:                              53|119                  (-55.4622%) [-2.24528x]
    Total read+write:                  970419|1082132              (-10.3234%) [-1.11512x]
    Estimated Cycles:                 1012949|1127018              (-10.1213%) [-1.11261x]
  Comparison with recvmmsg run:setup_nonblocking()
    Instructions:                      468571|587770               (-20.2799%) [-1.25439x]
    L1 Hits:                           882905|1071803              (-17.6243%) [-1.21395x]
    LL Hits:                            10191|10210                (-0.18609%) [-1.00186x]
    RAM Hits:                              57|119                  (-52.1008%) [-2.08772x]
    Total read+write:                  893153|1082132              (-17.4636%) [-1.21159x]
    Estimated Cycles:                  935855|1127018              (-16.9618%) [-1.20427x]
recv_cost::recv_cost::uring_multi run:setup_nonblocking()
  Instructions:                      628114|N/A                  (*********)
  L1 Hits:                          1145140|N/A                  (*********)
  LL Hits:                            11463|N/A                  (*********)
  RAM Hits:                             168|N/A                  (*********)
  Total read+write:                 1156771|N/A                  (*********)
  Estimated Cycles:                 1208335|N/A                  (*********)
  Comparison with dedicated run:setup_blocking()
    Instructions:                      454678|628114               (-27.6122%) [-1.38145x]
    L1 Hits:                           866202|1145140              (-24.3584%) [-1.32202x]
    LL Hits:                            10504|11463                (-8.36605%) [-1.09130x]
    RAM Hits:                             175|168                  (+4.16667%) [+1.04167x]
    Total read+write:                  876881|1156771              (-24.1958%) [-1.31919x]
    Estimated Cycles:                  924847|1208335              (-23.4610%) [-1.30652x]
  Comparison with epoll run:setup_nonblocking()
    Instructions:                      519312|628114               (-17.3220%) [-1.20951x]
    L1 Hits:                           960184|1145140              (-16.1514%) [-1.19263x]
    LL Hits:                            10182|11463                (-11.1751%) [-1.12581x]
    RAM Hits:                              53|168                  (-68.4524%) [-3.16981x]
    Total read+write:                  970419|1156771              (-16.1097%) [-1.19203x]
    Estimated Cycles:                 1012949|1208335              (-16.1699%) [-1.19289x]
  Comparison with recvmmsg run:setup_nonblocking()
    Instructions:                      468571|628114               (-25.4003%) [-1.34049x]
    L1 Hits:                           882905|1145140              (-22.8998%) [-1.29701x]
    LL Hits:                            10191|11463                (-11.0966%) [-1.12482x]
    RAM Hits:                              57|168                  (-66.0714%) [-2.94737x]
    Total read+write:                  893153|1156771              (-22.7891%) [-1.29515x]
    Estimated Cycles:                  935855|1208335              (-22.5500%) [-1.29116x]
  Comparison with uring run:setup_nonblocking()
    Instructions:                      587770|628114               (-6.42304%) [-1.06864x]
    L1 Hits:                          1071803|1145140              (-6.40420%) [-1.06842x]
    LL Hits:                            10210|11463                (-10.9308%) [-1.12272x]
    RAM Hits:                             119|168                  (-29.1667%) [-1.41176x]
    Total read+write:                 1082132|1156771              (-6.45236%) [-1.06897x]
    Estimated Cycles:                 1127018|1208335              (-6.72967%) [-1.07215x]
```

# Benchmark B - system impact

| backend     | ifaces | rate | sent  | recv  | lost | user_ms | sys_ms | vol_csw | invol_csw |
| ----------- | ------ | ---- | ----- | ----- | ---- | ------- | ------ | ------- | --------- |
| dedicated   | 1      | 1000 | 5000  | 5000  | 0    | 6.1     | 0.0    | 5000    | 0         |
| dedicated   | 1      | 2000 | 10000 | 10000 | 0    | 11.7    | 0.0    | 10000   | 0         |
| dedicated   | 1      | 4000 | 20000 | 20000 | 0    | 22.7    | 0.0    | 19996   | 0         |
| dedicated   | 2      | 1000 | 10000 | 10000 | 0    | 12.2    | 0.0    | 10000   | 0         |
| dedicated   | 2      | 2000 | 20000 | 20000 | 0    | 18.1    | 4.8    | 19999   | 0         |
| dedicated   | 2      | 4000 | 40000 | 40000 | 0    | 44.4    | 0.0    | 39997   | 0         |
| dedicated   | 4      | 1000 | 20000 | 20000 | 0    | 22.0    | 0.0    | 19999   | 0         |
| dedicated   | 4      | 2000 | 40000 | 40000 | 0    | 34.6    | 7.8    | 39993   | 12        |
| dedicated   | 4      | 4000 | 80000 | 80000 | 0    | 66.9    | 22.5   | 79956   | 48        |
| epoll       | 1      | 1000 | 5000  | 5000  | 0    | 3.9     | 3.9    | 4999    | 0         |
| epoll       | 1      | 2000 | 10000 | 10000 | 0    | 7.4     | 7.4    | 10000   | 0         |
| epoll       | 1      | 4000 | 20000 | 20000 | 0    | 14.2    | 14.3   | 19999   | 0         |
| epoll       | 2      | 1000 | 10000 | 9999  | 1    | 7.8     | 7.8    | 9865    | 0         |
| epoll       | 2      | 2000 | 20000 | 19999 | 1    | 14.6    | 14.6   | 19871   | 1         |
| epoll       | 2      | 4000 | 40000 | 39999 | 1    | 41.7    | 14.3   | 38664   | 1         |
| epoll       | 4      | 1000 | 20000 | 19997 | 3    | 26.8    | 0.0    | 16407   | 1         |
| epoll       | 4      | 2000 | 40000 | 39997 | 3    | 0.0     | 46.6   | 26749   | 62        |
| epoll       | 4      | 4000 | 80000 | 79997 | 3    | 0.0     | 103.7  | 66257   | 18        |
| recvmmsg    | 1      | 1000 | 5000  | 5000  | 0    | 0.0     | 7.9    | 5000    | 0         |
| recvmmsg    | 1      | 2000 | 10000 | 10000 | 0    | 0.0     | 15.1   | 10000   | 0         |
| recvmmsg    | 1      | 4000 | 20000 | 20000 | 0    | 0.0     | 28.7   | 19999   | 0         |
| recvmmsg    | 2      | 1000 | 10000 | 9999  | 1    | 0.0     | 15.4   | 9896    | 0         |
| recvmmsg    | 2      | 2000 | 20000 | 19999 | 1    | 0.0     | 29.6   | 19894   | 0         |
| recvmmsg    | 2      | 4000 | 40000 | 39999 | 1    | 0.0     | 57.9   | 39893   | 0         |
| recvmmsg    | 4      | 1000 | 20000 | 19997 | 3    | 0.0     | 26.2   | 15838   | 5         |
| recvmmsg    | 4      | 2000 | 40000 | 39997 | 3    | 0.0     | 52.7   | 32025   | 7         |
| recvmmsg    | 4      | 4000 | 80000 | 79997 | 3    | 0.0     | 101.0  | 63199   | 69        |
| uring       | 1      | 1000 | 5000  | 5000  | 0    | 0.0     | 7.3    | 5048    | 0         |
| uring       | 1      | 2000 | 10000 | 10000 | 0    | 0.0     | 14.0   | 10047   | 0         |
| uring       | 1      | 4000 | 20000 | 20000 | 0    | 0.0     | 26.7   | 20046   | 1         |
| uring       | 2      | 1000 | 10000 | 9999  | 1    | 0.0     | 14.2   | 9897    | 0         |
| uring       | 2      | 2000 | 20000 | 19999 | 1    | 0.0     | 27.2   | 19924   | 0         |
| uring       | 2      | 4000 | 40000 | 39999 | 1    | 7.6     | 44.8   | 39836   | 2         |
| uring       | 4      | 1000 | 20000 | 19997 | 3    | 3.8     | 20.3   | 14763   | 10        |
| uring       | 4      | 2000 | 40000 | 39997 | 3    | 8.1     | 42.2   | 33084   | 7         |
| uring       | 4      | 4000 | 80000 | 79997 | 3    | 15.3    | 78.8   | 61615   | 43        |
| uring_multi | 1      | 1000 | 5000  | 5000  | 0    | 1.0     | 6.1    | 5000    | 0         |
| uring_multi | 1      | 2000 | 10000 | 10000 | 0    | 1.7     | 11.3   | 10000   | 0         |
| uring_multi | 1      | 4000 | 20000 | 20000 | 0    | 3.9     | 21.1   | 19997   | 0         |
| uring_multi | 2      | 1000 | 10000 | 10000 | 0    | 1.3     | 7.0    | 5000    | 0         |
| uring_multi | 2      | 2000 | 20000 | 20000 | 0    | 2.5     | 13.4   | 9999    | 0         |
| uring_multi | 2      | 4000 | 40000 | 40000 | 0    | 4.8     | 25.6   | 19996   | 0         |
| uring_multi | 4      | 1000 | 20000 | 20000 | 0    | 1.8     | 9.4    | 5000    | 2         |
| uring_multi | 4      | 2000 | 40000 | 40000 | 0    | 13.6    | 9.5    | 9995    | 22        |
| uring_multi | 4      | 4000 | 80000 | 80000 | 0    | 35.1    | 8.6    | 19984   | 11        |

# Benchmark C - system contention

## 4 core ~75% utilization

| backend     | ifaces | rate | sent  | recv  | lost | user_ms | sys_ms | vol_csw | invol_csw |
| ----------- | ------ | ---- | ----- | ----- | ---- | ------- | ------ | ------- | --------- |
| dedicated   | 4      | 4000 | 79991 | 79989 | 2    | 7.8     | 43.5   | 61858   | 169       |
| epoll       | 4      | 4000 | 79997 | 79994 | 3    | 5.7     | 40.7   | 33651   | 327       |
| recvmmsg    | 4      | 4000 | 79995 | 79994 | 1    | 6.3     | 40.3   | 34500   | 389       |
| uring       | 4      | 4000 | 80000 | 79997 | 3    | 3.4     | 46.7   | 39036   | 284       |
| uring_multi | 4      | 4000 | 79993 | 79992 | 1    | 4.4     | 20.7   | 11021   | 110       |

## 4 core ~90% utilization

| backend     | ifaces | rate | sent  | recv  | lost | user_ms | sys_ms | vol_csw | invol_csw |
| ----------- | ------ | ---- | ----- | ----- | ---- | ------- | ------ | ------- | --------- |
| dedicated   | 4      | 4000 | 79991 | 79991 | 0    | 8.1     | 27.0   | 56873   | 81        |
| epoll       | 4      | 4000 | 79993 | 79991 | 2    | 9.9     | 31.7   | 40314   | 150       |
| recvmmsg    | 4      | 4000 | 79993 | 79991 | 2    | 7.1     | 33.7   | 39261   | 184       |
| uring       | 4      | 4000 | 80000 | 79991 | 9    | 7.9     | 29.2   | 37673   | 115       |
| uring_multi | 4      | 4000 | 79993 | 79992 | 1    | 3.6     | 16.2   | 9232    | 64        |

**NOTE:** Fewer involuntary context switches under higher CPU utilization is counter intuitive, but
correct. It means the receiver is being starved rather than interrupted. Compare the sys_ms kernel
CPU time.

# Takeaways

* The pure CPU cost of the receive backends don't matter hugely, because the dominant cost is the
  syscalls and context switching
* The multiplex methods are all pretty close to each other in terms of results
* It appears all backends degrade nicely when the system is under high CPU load
* It doesn't look like I'm going to get absolutely no dropped frames
* Batching receives in the multishot backend dramatically reduces kernel CPU time and context
  switches, moreso than the other multiplex backends, and even at high CPU load
