```
oresults-connector 1.1.0
Otakar Hirš <tech@oresults.eu>
Tool for automatic upload of start list and result files to Oresutls.eu

Supply an "API key" you get from event settings and specify "path" to folder, that will be
recursivelly watched for file changes. Recognized iof xml ResultList and StartList files will get
automaticly uploaded on change.

USAGE:
    oresults-connector [OPTIONS] --key <API_KEY>

OPTIONS:
    -h, --help                    Print help information
    -k, --key <API_KEY>           API key provided in event settings
    -p, --path <PATH_TO_FILES>    Path to folder (or a single file) recursively watched for changes
    -V, --version                 Print version information
```
