# OResults connector
Tool for automatic upload of start list and result files to OResults. It uploads files from a given folder as they are created or modified. 

Part of the [OResults](https://oresults.eu) platform. See [docs for organizers](https://docs.oresults.eu) for more details.

### Download 
See [Releases](https://github.com/oresults/oresults-connector/releases).

### Usage
`oresults-connector --key 48qh1d31hd1 --path ./folder_with_xml_files`

```
oresults-connector 1.2.0
Otakar Hirš <tech@oresults.eu>
Tool for automatic upload of start list and result files to OResults.

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

---
# Use integrated services instead
### QuickEvent
If you use [QuickEvent](https://github.com/Quick-Event/quickbox), you can use the newly integrated OResults Connector service. It is available since release version [2.6.5](https://github.com/Quick-Event/quickbox/releases/tag/QE-v2.6.5).

### MeOS
If you use [MeOs](https://www.melin.nu/meos/en/), you can use the MeOS Online Protocol XML v2 in the services menu. The api endpoint is `https://api.oresults.eu/meos`, use the event API key as the password. (ZIP copression is not supported)
