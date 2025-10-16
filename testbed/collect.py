import csv
import os
import tempfile
import time
from datetime import datetime, timedelta
from io import TextIOWrapper

from influxdb_client import InfluxDBClient  # type: ignore


def query_chunk_with_retry(
    client, bucket, measurement_name, start_time, end_time, max_retries=1
):
    """Query a time chunk with retry logic"""
    query = f"""from(bucket:"{bucket}")
                |> range(start: {start_time}, stop: {end_time})
                |> filter(fn: (r) => r._measurement == "{measurement_name}")
                """

    for attempt in range(max_retries + 1):
        try:
            tables = client.query_api().query_csv(query)
            return tables.to_values()
        except Exception as e:
            if attempt >= max_retries:
                raise e
            time.sleep(10)


def worker(queue, addresses, token, bucket, org, measurement_name):
    with tempfile.NamedTemporaryFile(delete=False) as tmpfile:
        with TextIOWrapper(tmpfile, encoding="utf-8") as file:
            writer = csv.writer(file, delimiter="\t")
            write_headers = True

            # Get current time and calculate start time (assuming we want last 24 hours of data)
            end_time = datetime.now()
            # TODO: Make this configurable
            start_time = end_time - timedelta(hours=1)
            chunk_duration = timedelta(minutes=1)

            for address in addresses:
                with InfluxDBClient(
                    url="http://" + address, token=token, org=org
                ) as client:
                    current_start = start_time

                    while current_start < end_time:
                        current_end = min(current_start + chunk_duration, end_time)

                        start_str = current_start.strftime("%Y-%m-%dT%H:%M:%SZ")
                        end_str = current_end.strftime("%Y-%m-%dT%H:%M:%SZ")

                        values = query_chunk_with_retry(
                            client, bucket, measurement_name, start_str, end_str
                        )

                        if len(values) < 3:
                            current_start = current_end
                            continue
                        values = values[3:]

                        if len(values) == 0:
                            current_start = current_end
                            continue

                        header = values[0]
                        if write_headers:
                            writer.writerow(header)
                            write_headers = False

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

                        current_start = current_end

        # tmpfile.close()  # Close before sending to threads
        print("spans done,", tmpfile.name)
        queue.put((measurement_name, tmpfile.name))


def listener(queue, filepath):
    import shutil

    arkdir = f"{filepath}.d"
    os.mkdir(arkdir)
    while True:
        m = queue.get()
        print(m)
        if m == "kill":
            break

        metrixName, tmpfile_name = m
        new_file = shutil.copy(tmpfile_name, arkdir)
        os.rename(
            new_file,
            os.path.join(arkdir, f"{metrixName}.csv"),
        )

        # try:
        #     with tarfile.open(filepath, "w:xz") as tar:  # type: ignore
        #         tar.add(tmpfile_name)
        # except Exception:
        #     pass

    # with tarfile.open(filepath, "w:zst") as tar:  # type: ignore
    #     while True:
    #         m = queue.get()
    #         if m == "kill":
    #             break
    #
    # metrixName, tmpfile_name = m
    # tar.add(tmpfile_name)

    # tarinfo = tarfile.TarInfo(f"{metrixName}.csv")
    # tarinfo.size = os.path.getsize(tmpfile_name)
    # tarinfo.mtime = int(os.path.getmtime(tmpfile_name))
    # with open(tmpfile_name, "rb") as tmpfile:
    #     tar.addfile(tarinfo, tmpfile)

    # os.remove(tmpfile_name)
