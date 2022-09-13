# OResults connector
Tool for automatic upload of start list and result files to OResutls.eu  
Files in given folder are uploaded on creation or change.  
Part of the [OResults](https://oresults.eu) platform. See [docs for organizers](https://docs.oresults.eu) for more details.

### Download 
See [Releases](https://github.com/oresults/oresults-connector/releases).

### Usage
`oresults-connector --key 48qh1d31hd1 --path ./folder_with_xml_files`

---
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

### QuickEvent
ps: If you are using ![QuickEvent](https://github.com/Quick-Event/quickbox), you can use integrated service instead. It is not available in the released version yet, but was sucessfully tested on local race. See https://github.com/Quick-Event/quickbox/actions/runs/2951271536 for download link.
