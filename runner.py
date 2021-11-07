import subprocess
import sys
from argparse import ArgumentParser
from pathlib import Path

TEST_CMD = ["cargo", "run", "--release", "--bin", "tester"]

if __name__ == "__main__":
    arg_parser = ArgumentParser()
    arg_parser.add_argument("exe", metavar="X")
    arg_parser.add_argument("tool_dir", metavar="TOOL")
    arg_parser.add_argument("in_dir", metavar="IN")
    arg_parser.add_argument("out_dir", metavar="OUT")
    arg_parser.add_argument("--verbose", action="store_true")
    args = arg_parser.parse_args()

    exe_path = Path(args.exe).resolve()
    tool_path = Path(args.tool_dir).resolve()
    cmd = TEST_CMD + [exe_path.as_posix()]

    in_dir = Path(args.in_dir)
    out_dir = Path(args.out_dir)
    if not out_dir.is_dir():
        out_dir.mkdir()

    total_score = 0
    case_cnt = 0
    for in_file in sorted(in_dir.iterdir()):
        res = subprocess.run(cmd, stdin=in_file.open(), capture_output=True, cwd=tool_path)
        score = int(res.stderr.decode("utf-8").split()[-1])
        if args.verbose:
            print(res.stderr.decode("utf-8"), file=sys.stderr)
        else:
            print(f"Input = '{in_file.name}', Score = {score}\r", end="")

        total_score += score
        case_cnt += 1

        out_file = out_dir / in_file.name
        out_file.write_bytes(res.stdout)
    print()
    print(f"Total score with {case_cnt} cases = {total_score}")
    print(f"Estimated score with 50 cases = {int(total_score / (case_cnt / 50))}")
