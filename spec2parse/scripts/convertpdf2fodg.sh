#!/bin/sh

# convert PDF file into collection of single page FODG files via Libreoffice and poppler-utils

# Example how to call
# script.sh Trusted-Platform-Module-2.0-Library-Part-2-Structures_Version-185_pub.pdf tpm185-part2

if [ $# -lt 1 ] || [ $# -gt 2 ]; then
    echo "Convert a PDF file (.pdf) into multiple single page FODG files (.xxxx.fodg) via Libreoffice"
    echo "Usage: $0 <input_pdf_file> [output_fodg_file_prefix]"
    exit 1
fi


START_TIME=$(date +%s)
INPUT_FILE="$1"

# create output filename from input filename
if [ $# -eq 2 ]; then
    OUTPUT_PREFIX="$2"
else
    OUTPUT_PREFIX="${INPUT_FILE%.pdf}"
fi

# sanity check
if [ ! -f "$INPUT_FILE" ]; then
    echo "Error: Input file '$INPUT_FILE' does not exist or is not a regular file."
    exit 1
fi

if ! file --mime-type "$INPUT_FILE" 2>/dev/null | grep -q "application/pdf"; then
    echo "Error: Input file '$INPUT_FILE' is not a PDF file."
    exit 1
fi

# check if required commands are installed
for cmd in pdfseparate pdfinfo; do
    if ! command -v "$cmd" &> /dev/null; then
        echo "Error: Command '$cmd' is not installed or not in the PATH" >&2
        echo "Please install poppler-utils package (or similar package that contains these)" >&2
    exit 1
fi
done

# split into input.pdf
echo -n "Extracting single PDF pages from $INPUT_FILE... "
pdfseparate "$INPUT_FILE" "${OUTPUT_PREFIX}.%04d.pdf"

# sanity check number of pages
PAGE_COUNT=$(pdfinfo "$INPUT_FILE" | grep Pages | awk '{print $2}')
echo "$PAGE_COUNT pages"

if [ "$PAGE_COUNT" -gt 9999 ]; then
    echo "Error: this PDF file seems to have more than 9999 pages, which exceeds the maximum supported by this script."
    exit 1
fi

# note: libreoffice REALLY does not like more than one instance running,
# so this has to be done one page after another...
echo "Converting each page to fodg... "
for i in "${OUTPUT_PREFIX}".*.pdf; do
        libreoffice --headless --draw --convert-to "fodg:OpenDocument Drawing Flat XML" "$i"
done


END_TIME=$(date +%s)
RUNTIME=$((END_TIME - START_TIME))
MINUTES=$((RUNTIME / 60))
SECONDS=$((RUNTIME % 60))
printf "...converted $PAGE_COUNT pages (in %dm%02ds)\n" $MINUTES $SECONDS
