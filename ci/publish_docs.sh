#!/bin/bash

GITHUB=mini-fs
DOCS_DIR=.pages/

cargo doc --all-features
rm -rf $DOCS_DIR
mv target/doc $DOCS_DIR

cat << EOF > $DOCS_DIR/index.html
<!DOCTYPE html>
<html>
<head>
<meta http-equiv="refresh" content="0; url=mini_fs/index.html" />
</head>
<body>
You are being redirected to the <a href="mini_fs/index.html">documentation page</a>...
</body>
</html>
EOF

git -C $DOCS_DIR init && \
git -C $DOCS_DIR remote add origin https://germangb:$TOKEN@github.com/germangb/$GITHUB.git && \
git -C $DOCS_DIR checkout -b gh-pages && \
git -C $DOCS_DIR add -A && \
git -C $DOCS_DIR commit -m "Publish docs" > /dev/null && \
git -C $DOCS_DIR push origin gh-pages --force --quiet