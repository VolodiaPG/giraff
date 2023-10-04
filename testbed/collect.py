import csv
import os
import tarfile
import tempfile
from io import TextIOWrapper

from influxdb_client import InfluxDBClient  # type: ignore


def worker(queue, addresses, token, bucket, org, measurement_name):
    with tempfile.NamedTemporaryFile(delete=False) as tmpfile:
        with TextIOWrapper(tmpfile, encoding="utf-8") as file:
            writer = csv.writer(file, delimiter="\t")
            write_headers = True
            for address in addresses:
                with InfluxDBClient(
                    url="http://" + address, token=token, org=org
                ) as client:
                    query = f"""from(bucket:"{bucket}") 
                                |> range(start: 0) 
                                |> filter(fn: (r) => r._measurement == "{measurement_name}")
                                """
                    tables = client.query_api().query_csv(query)
                    values = tables.to_values()

                    if len(values) < 3:
                        continue
                    values = values[3:]

                    header = values[0]
                    if write_headers:
                        writer.writerow(header)
                        write_headers = False

                    if len(values) == 0:
                        continue
                    values = values[1:]

                    remove_next = False
                    for row in values:
                        if row[0].startswith("#"):
                            remove_next = True
                            continue

                        if remove_next:
                            remove_next = False
                            continue

                        writer.writerow(row)

        tmpfile.close()  # Close before sending to threads
        queue.put((measurement_name, tmpfile.name))


def listener(queue, filepath):
    with tarfile.open(filepath, "w:xz", preset=9) as tar:
        while True:
            m = queue.get()
            if m == "kill":
                break

            metrixName, tmpfile_name = m

            tarinfo = tarfile.TarInfo(f"{metrixName}.csv")
            tarinfo.size = os.path.getsize(tmpfile_name)
            tarinfo.mtime = os.path.getmtime(tmpfile_name)
            with open(tmpfile_name, "rb") as tmpfile:
                tar.addfile(tarinfo, tmpfile)

            os.remove(tmpfile_name)
