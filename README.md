![Build](https://github.com/twosigma/gcsthin/workflows/Build/badge.svg)

gcsthin
=======

High performance streaming upload/download tool for Google Cloud Storage (GCS).

# Usage

Authentication must be set either with:
1) The `GOOGLE_APPLICATION_CREDENTIALS` environment variable must be set to the
   service account json access file when invoking gcsthin.
2) If the above env var is not set, gcsthin uses the compute metadata token
   available when running on GCE.

### Upload

gcsthin reads content from `STDIN` and writes it to `GS_URL`.

```
gcsthin cp - GS_URL < file
```

### Download

gcsthin reads content from `GS_URL` and writes it to `STDOUT`.

```
gcsthin cp GS_URL - > file
```

# Benchmarks

We tested different programs to transfer a 5 GB stream to GCS.
Each program is run through `/usr/bin/time`. The CPU usage column is the sum of
the reported user and system CPU time divided by 5 GB. The max speed is computed
using the CPU usage. Reported numbers show the median of 3 runs.

Other tested programs:
* **gsutil**: Google's official python CLI tool
* **gcsgo**: Tool built using the official Google Go Library
* **curl**: Widely used CLI tool written in C
* **wget**: Widely used CLI tool. Does not support streaming uploads

### Upload

Name                  | CPU usage     | Max speed, single CPU  | Memory use   |
--------------------- | ------------- | ---------------------- | ------------ |
**gcsthin (OpenSSL)** | 1.2 CPUsec/GB | 800 MB/s               | 9 MB         |
gcsthin (rustls)      | 1.7 CPUsec/GB | 590 MB/s               | 6 MB         |
gsutil                | 6.3 CPUsec/GB | 160 MB/s               | 249 MB       |
gcsgo                 | 2.6 CPUsec/GB | 390 MB/s               | 39MB         |
curl                  | 2.9 CPUsec/GB | 350 MB/s               | 11 MB        |
wget                  | N/A           | N/A                    | N/A          |

### Download

Name                   | CPU usage      | Max speed, single CPU  | Memory use   |
---------------------- | -------------- | ---------------------- | ------------ |
**gcsthin (OpenSSL)**  | 1.3 CPUsec/GB  | 770 MB/s               | 9 MB         |
gcsthin (rustls)       | 1.8 CPUsec/GB  | 560 MB/s               | 6 MB         |
gsutil                 | 2.8 CPUsec/GB  | 360 MB/s               | 42 MB        |
gcsgo                  | 3.6 CPUsec/GB  | 280 MB/s               | 34 MB        |
curl                   | 1.8 CPUsec/GB  | 560 MB/s               | 12 MB        |
wget                   | 5.0 CPUsec/GB  | 200 MB/s               | 10 MB        |


## Acknowledgments
* Author: Nicolas Viennot [@nviennot](https://github.com/nviennot)
* Reviewer: Vitaly Davidovich [@vitalyd](https://github.com/vitalyd)
* Developed as a [Two Sigma Open Source](https://opensource.twosigma.com) initiative

## License
gcsthin is licensed under the
[Apache 2.0 license](https://www.apache.org/licenses/LICENSE-2.0).
