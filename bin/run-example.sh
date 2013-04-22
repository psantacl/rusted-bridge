set -x 
set -e
set -u 
time $(dirname $0)/../rust/bin/rusted-bridge clj-ls ~/ | $(dirname $0)/../rust/bin/rusted-bridge clj-grep screen | $(dirname $0)/../rust/bin/rusted-bridge clj-wc-l
