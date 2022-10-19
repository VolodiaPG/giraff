import codecs
import csv
import sys
import tarfile
from io import BytesIO
from datetime import datetime
import os
from integration import aliases

import requests

if __name__ == "__main__":
    URL = f"http://localhost:{os.environ['port']}"

    def GetMetrixNames(url):
        response = requests.get('{0}/api/v1/label/__name__/values'.format(url))
        names = response.json()['data']  # Return metrix names
        return names


    """
    Prometheus hourly data as csv.
    """

    if len(sys.argv) != 1:
        print(f'Usage: {sys.argv[0]}\n use port env variable to set the port on localhost')
        sys.exit(1)
    metrixNames = GetMetrixNames(URL)

    today = datetime.today()

    today = today.strftime("%Y-%m-%d-%H-%M")
    archive = f"metrics_{today}.tar.gz"

    with tarfile.open(archive, "w:gz") as tar:
        names = aliases()
        with BytesIO() as tmpfile:
            codecinfo = codecs.lookup("utf8")
            file = codecs.StreamReaderWriter(tmpfile, codecinfo.streamreader, codecinfo.streamwriter)
            writer = csv.writer(file, delimiter="\t")
            writer.writerow(["instance", "name"])
            for (key, value) in names.items():
                writer.writerow([key, value])
            tarinfo = tarfile.TarInfo("names.csv")
            tarinfo.size = tmpfile.tell()
            tmpfile.seek(0)
            tar.addfile(tarinfo, tmpfile)

        for metrixName in metrixNames:
            with BytesIO() as tmpfile:
                # Create a wrapper around the in-memory file in order to encode to utf-8 strings instead of bytes
                codecinfo = codecs.lookup("utf8")
                file = codecs.StreamReaderWriter(tmpfile, codecinfo.streamreader, codecinfo.streamwriter)
                writer = csv.writer(file, delimiter="\t")

                # now its hardcoded for hourly
                response = requests.get('{0}/api/v1/query'.format(URL),
                                        params={'query': metrixName + f'[{os.environ["period"]}]'})
                results = response.json()['data']['result']
                # Build a list of all labelnames used.
                # gets all keys and discard __name__
                labelnames = set()
                for result in results:
                    labelnames.update(result['metric'].keys())
                # Canonicalize
                labelnames.discard('__name__')
                labelnames = sorted(labelnames)

                writer.writerow(['name'] + labelnames + ['timestamp', 'value'])
                for result in results:
                    for label in labelnames:
                        l = list(result['metric'].values())
                        for value in result['values']:
                            writer.writerow(l + value)

                tarinfo = tarfile.TarInfo(f"{metrixName}.csv")
                tarinfo.size = tmpfile.tell()
                tmpfile.seek(0)
                tar.addfile(tarinfo, tmpfile)

    try:
        os.remove("latest_metrics.tar.gz")
    except (FileExistsError, FileNotFoundError):
        pass
    os.symlink(archive, "latest_metrics.tar.gz")
