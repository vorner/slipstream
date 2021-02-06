window.BENCHMARK_DATA = {
  "lastUpdate": 1612642755135,
  "repoUrl": "https://github.com/vorner/slipstream",
  "entries": {
    "Track benchmarks": [
      {
        "commit": {
          "author": {
            "name": "vorner",
            "username": "vorner"
          },
          "committer": {
            "name": "vorner",
            "username": "vorner"
          },
          "id": "9a41f1b02bc7a9b02f967d01ff4d8b6af512b905",
          "message": "Migrate to GH actions",
          "timestamp": "2021-01-13T21:55:02Z",
          "url": "https://github.com/vorner/slipstream/pull/6/commits/9a41f1b02bc7a9b02f967d01ff4d8b6af512b905"
        },
        "date": 1612642753409,
        "tool": "cargo",
        "benches": [
          {
            "name": "sum_vec",
            "value": 64539,
            "range": "± 6488",
            "unit": "ns/iter"
          },
          {
            "name": "sum_scalar",
            "value": 378200,
            "range": "± 18491",
            "unit": "ns/iter"
          },
          {
            "name": "dot_product_vec",
            "value": 97669,
            "range": "± 7499",
            "unit": "ns/iter"
          },
          {
            "name": "dot_product_scalar",
            "value": 380194,
            "range": "± 26939",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}