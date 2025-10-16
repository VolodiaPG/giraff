import os
import subprocess

archive = "toto.tar.xz"

env = os.environ.copy()
env["XZ_OPT"] = "-9"

args = ["tar", "--remove-files", "-cvJf", archive, "-C", f"{archive}.d/", "."]
# args = f"tar cf - {archive}.d/ | xz -4e > {archive}"
# args = f"tar -cf file.tar -C {archive}.d/ ."
print(args)

# print(" ".join(args))
# print(" ".join(args))

subprocess.run(
    args,
    env=env,
    # shell=True,
)
