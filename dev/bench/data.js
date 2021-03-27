window.BENCHMARK_DATA = {
  "lastUpdate": 1616869163502,
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
      },
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
          "id": "69db55a92945ebb2adf262691c7b638f75679a51",
          "message": "Migrate to GH actions",
          "timestamp": "2021-01-13T21:55:02Z",
          "url": "https://github.com/vorner/slipstream/pull/6/commits/69db55a92945ebb2adf262691c7b638f75679a51"
        },
        "date": 1612642856221,
        "tool": "cargo",
        "benches": [
          {
            "name": "sum_vec",
            "value": 68066,
            "range": "± 3021",
            "unit": "ns/iter"
          },
          {
            "name": "sum_scalar",
            "value": 500431,
            "range": "± 23862",
            "unit": "ns/iter"
          },
          {
            "name": "dot_product_vec",
            "value": 109730,
            "range": "± 4308",
            "unit": "ns/iter"
          },
          {
            "name": "dot_product_scalar",
            "value": 499253,
            "range": "± 17635",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "vorner@vorner.cz",
            "name": "Michal 'vorner' Vaner",
            "username": "vorner"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f47ef665d80529b54a59fdaa3b7a4aa76517f237",
          "message": "Merge pull request #6 from vorner/constgen\n\nMigrate to GH actions",
          "timestamp": "2021-02-06T21:24:38+01:00",
          "tree_id": "ccf4fe070645219401d05de7f3aa678e856e8579",
          "url": "https://github.com/vorner/slipstream/commit/f47ef665d80529b54a59fdaa3b7a4aa76517f237"
        },
        "date": 1612643326779,
        "tool": "cargo",
        "benches": [
          {
            "name": "sum_vec",
            "value": 68719,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "sum_scalar",
            "value": 548471,
            "range": "± 253",
            "unit": "ns/iter"
          },
          {
            "name": "dot_product_vec",
            "value": 128910,
            "range": "± 1862",
            "unit": "ns/iter"
          },
          {
            "name": "dot_product_scalar",
            "value": 548595,
            "range": "± 3102",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "vorner@vorner.cz",
            "name": "Michal 'vorner' Vaner",
            "username": "vorner"
          },
          "committer": {
            "email": "vorner@vorner.cz",
            "name": "Michal 'vorner' Vaner",
            "username": "vorner"
          },
          "distinct": true,
          "id": "f99856052efde95f9608a2849b4b7ab3cccfdd14",
          "message": "Fix packed_simd",
          "timestamp": "2021-02-13T16:37:09+01:00",
          "tree_id": "5bf885762f7ccde4c63d44e56fc6b8b2601ca1fa",
          "url": "https://github.com/vorner/slipstream/commit/f99856052efde95f9608a2849b4b7ab3cccfdd14"
        },
        "date": 1613230944041,
        "tool": "cargo",
        "benches": [
          {
            "name": "sum_vec",
            "value": 82693,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "sum_scalar",
            "value": 659275,
            "range": "± 2190",
            "unit": "ns/iter"
          },
          {
            "name": "dot_product_vec",
            "value": 112744,
            "range": "± 8361",
            "unit": "ns/iter"
          },
          {
            "name": "dot_product_scalar",
            "value": 659294,
            "range": "± 860",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "vorner@vorner.cz",
            "name": "Michal 'vorner' Vaner",
            "username": "vorner"
          },
          "committer": {
            "email": "vorner@vorner.cz",
            "name": "Michal 'vorner' Vaner",
            "username": "vorner"
          },
          "distinct": true,
          "id": "a94fd7656fe98eea3a270f2884b1809a91e19a06",
          "message": "Clippy-suggested simplifications",
          "timestamp": "2021-03-27T19:14:47+01:00",
          "tree_id": "f890892b2aa942faf17b04f447d41ebd21e73608",
          "url": "https://github.com/vorner/slipstream/commit/a94fd7656fe98eea3a270f2884b1809a91e19a06"
        },
        "date": 1616869162852,
        "tool": "cargo",
        "benches": [
          {
            "name": "sum_vec",
            "value": 76933,
            "range": "± 3164",
            "unit": "ns/iter"
          },
          {
            "name": "sum_scalar",
            "value": 612556,
            "range": "± 22918",
            "unit": "ns/iter"
          },
          {
            "name": "dot_product_vec",
            "value": 143005,
            "range": "± 2322",
            "unit": "ns/iter"
          },
          {
            "name": "dot_product_scalar",
            "value": 620197,
            "range": "± 19689",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}