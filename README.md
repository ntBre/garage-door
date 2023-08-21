# garage-door
like a portal but a lot bigger

# Usage

`garage-door` is a tool for quickly retrieving data sets from QCFractal servers
and converting the results to a sequence of QCPortal [Record]s and OpenFF
[Molecule]s like a call to [openff.toolkit.BasicResultCollection.to_records].
Well, it almost does that, except for the fact that constructing a `Molecule`
outside of the Python code is difficult. So it actually prints `Record`s and the
building blocks for the corresponding `Molecule`s in JSON format for further
processing by Python. See [shim.py](python/shim.py) for an example of that.

There are two subcommands for `garage-door`: `get` and `convert`. The first of
these fetches a named dataset from QCFractal:

``` shell
garage-door get "OpenFF multiplicity correction torsion drive data v1.1" \
	    --dataset-type TorsionDrive
```

Unfortunately, I haven't figured out a good way to detect the type of the
dataset, so the `--dataset-type` flag is required for now.

The `convert` subcommand instead reads an existing dataset file with contents
like

``` json
{
  "entries": {
    "https://api.qcarchive.molssi.org:443/": [
      {
        "type": "torsion",
        "record_id": "107276605",
        "cmiles": "[H:12][c:1]1[c:2]([c:3]([c:4]([c:5]([c:6]1[H:16])[H:15])[S:7](=[O:8])(=[O:9])[N:10]([H:17])[N:11]([H:18])[H:19])[H:14])[H:13]",
        "inchi_key": "VJRITMATACIYAF-UHFFFAOYNA-N"
      }
    ]
  }
}
```

and then runs the rest of the code.

# Benchmarks

Why would you use this? In short, calling `to_records` in Python is very
expensive for some reason that I haven't been able to determine yet, and the
resulting `Record` types are not immediately JSON serializable for some reason.
This means that a script depending on a call to `to_records` has to make an
expensive call every time and wait forever on the results. This package shows
both that it shouldn't take so long *and* that the results can trivially be
serialized and deserialized to JSON. Without further ado, some numbers:

| Dataset                 | Python              | Rust               | Shim               |
|-------------------------|---------------------|--------------------|--------------------|
| testfiles/core-opt.json | 4.325 s ±  0.370 s  | 2.164 s ±  0.530 s | 1.567 s ±  0.036 s |
| testfiles/core-td.json  | 16.316 s ±  0.676 s | 4.502 s ±  0.477 s | 1.134 s ±  0.021 s |
| filtered-industry.json  | RUNNING             | 71.4 s             | 222.5 s            |

The Python numbers are taken from running either `hyperfine "python opt.py"` for
optimization data sets or `hyperfine "python td.py"` for torsion drive data
sets. The Rust results are from `hyperfine target/release/garage-door convert
testfiles/core-opt.json -d Optimization` or the TorsionDrive analogue. For a
fair comparison, the Rust results should be added to the estimates for running
`hyperfine "python shim.py /tmp/opt.json"` or the torsion drive equivalent, in
the last column. The Rust timings are also inflated by writing the results to
stdout, but that's somewhat unavoidable, given it has to communicate with Python
somehow.

The `filtered-industry.json` dataset has more than 70,000 records, so the
results are not run through `hyperfine`. The command for the Python result is
simply

``` shell
time python python/industry.py
```

and those for the Rust version are

``` shell
data=../../projects/benchmarking/datasets/filtered-industry.json
target/release/garage-door convert $data -d Optimization > /tmp/industry.json
time python python/shim.py /tmp/industry.json
```

## Conclusions

The Rust version plus the Python shim are comparable to, but a bit faster than,
the pure Python version for a very small torsion drive dataset but approximately
3 times faster for a small optimization dataset. I expect the difference to be
much more pronounced for the industry dataset, when I get the results.

I haven't done any profiling on the Python shim itself, so there is likely a
better way to turn the JSON into Python objects. I plan to look into
[PyO3](https://pyo3.rs/v0.19.2/) to see if I can convert to Python objects
directly (and wrap this up in a nice Python module). If not, a more compact
serialization format may lead to significant improvements over JSON. `serde`
makes that trivial from the Rust side, so as long as a format is available in
Python, I can try it out pretty easily.

[Record]: https://github.com/MolSSI/QCPortal/blob/ff6f8bdf733b648e927223c89126a3ba37f88b69/qcportal/models/records.py#L251
[Molecule]: https://docs.openforcefield.org/projects/toolkit/en/latest/api/generated/openff.toolkit.topology.Molecule.html#openff.toolkit.topology.Molecule
[openff.toolkit.BasicResultCollection.to_records]: https://docs.openforcefield.org/projects/qcsubmit/en/latest/api/generated/openff.qcsubmit.results.BasicResultCollection.html?highlight=to_records#openff.qcsubmit.results.BasicResultCollection.to_records
