import argparse
import subprocess
from pathlib import Path

import pandas as pd


def main():
    parser = argparse.ArgumentParser(
        prog="dump_all",
        description="Iterates through all kpep files it can identify, and dump the output of `apmc list` for that cpu skew",
    )
    parser.add_argument("--kpep-base", default="/usr/share/kpep", type=Path)
    parser.add_argument("--out", default=".", type=Path)
    args = parser.parse_args()
    kpep_base: Path = args.kpep_base
    outpath: Path = args.out

    if (not kpep_base.exists()) or (not kpep_base.is_dir()):
        raise NotADirectoryError("kpep base doesn't exist or isn't a directory")
    if (not outpath.exists()) or (not outpath.is_dir()):
        raise NotADirectoryError("out directory doesn't exist or isn't a directory")

    kpep_files = [f.name for f in kpep_base.iterdir() if f.name.startswith("cpu_")]

    families = pd.read_csv("family_names.csv")
    for index, row in families.iterrows():
        name = row["name"]
        cpu_type = row["cpu_type"]
        cpu_subtype = row["cpu_subtype"]
        cpu_family = row["cpu_family"]

        dump_file = outpath / f"{name}.txt"
        kpep_file = kpep_base / f"cpu_{cpu_type}_{cpu_subtype}_{cpu_family}.plist"

        if kpep_file.name in kpep_files:
            kpep_files.remove(kpep_file.name)
        else:
            print(f"Missing kpep file for {name}: {kpep_file.absolute()}")
            continue

        with open(dump_file, "w") as stdout:
            subprocess.run(
                ["target/debug/apmc", "list", str(kpep_file.absolute())],
                stdout=stdout,
                check=True,
            )

    for kpep_file in kpep_files:
        print(f"Unknown kpep file: {(kpep_base / kpep_file).absolute()}")


if __name__ == "__main__":
    main()
