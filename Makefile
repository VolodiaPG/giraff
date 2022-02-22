DOC?=README

all: README

## Requires the mermaid filter available with `npm install --global mermaid-filter`

pdf:$(DOC).MD
	mkdir -p out
	export VERSION=`git describe --tags --abbrev=0` && \
	export NAME=`cat $(DOC).MD | sed -n 's/^\s*title: "\(.*\)"$$/\1/p' | sed 's/ /_/g'` && \
	sed -e "s/##VERSION##/$${VERSION}/g" $(DOC).MD | \
	pandoc -F mermaid-filter -t pdf -s -o out/$${NAME}.pdf --template eisvogel --listings

README:
	make pdf
