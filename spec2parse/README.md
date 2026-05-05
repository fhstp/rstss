
# Spec2Parse

*spec2parse* is a command-line tool that implements a custom parser for the official TPM 2.0 specification in PDF, as published by the Trusted Computing Group (TCG). The official TCG specifications are the "ground truth". A (semi-)automated extraction of the information from the PDFs into a structured output format eases the subsequent use of this information for code/implementations, possibly in different programming languages.

Note that it is NOT the goal to implement a 100% automated/perfect process of "PDF (table) data to ready-to-use code snippets". Rather, a semi-automated extraction process reduces the chance of human errors, additional manual work is still needed.


### 1) Download source specifications

**Trusted Platform Module 2.0** Library, Version 185 – March 2026 \
https://trustedcomputinggroup.org/resource/tpm-library-specification/ \
Trusted-Platform-Module-2.0-Library-Part-2-Structures_Version-185_pub.pdf \
Trusted-Platform-Module-2.0-Library-Part-3-Commands_Version-185_pub.pdf

There are supplemental specifications of interest, table extraction is not fully implemented.

**TCG Algorithm Registry** Version 2.0 - July 28, 2025 \
https://trustedcomputinggroup.org/resource/tcg-algorithm-registry/ \
TCG-Algorithm-Registry-Version-2.0_pub.pdf

**TCG TPM Vendor ID Registry** Family 1.2 and 2.0 Version 1.07 Revision 0.02 - November 20, 2024 \
https://trustedcomputinggroup.org/resource/vendor-id-registry/ \
TCG-TPM-Vendor-ID-Registry-Family-1.2-and-2.0-Version-1.07-Revision-0.02_pub.pdf

**Registry of Reserved TPM 2.0 Handles and Localities** Version 1.2 Revision 1.00 - November 2, 2023 \
https://trustedcomputinggroup.org/resource/registry/ \
Registry-of-Reserved-TPM-2.0-Handles-and-Localities-Version-1.2-Revision-1.00_pub.pdf


### 2) Conversion PDF to FODG

The first step step is to split a PDF file into individual pages, then convert them each from PDF to FODG via poppler-utils and Libreoffice.
Libreoffice encodes into the XML-based FODG format the objects on the page.

The provided `convertpdf2fodg.sh` script is to be called by specifying a `inputfile.pdf` filename and the name prefix of the FODG files to be created `outputname` (.xxxx.fodg). \
For example: `convertpdf2fodg.sh Trusted-Platform-Module-2.0-Library-Part-2-Structures_Version-185_pub.pdf tpm185-part2`

This process takes some time (as Libreoffice allows only one instance/page to run at a time).

The process was tested/performce using an "all defaults" install of Ubuntu 26.0, in a VM. The output/result may slightly differ, depending on installed additional fonts and configured locale -- this has not been widely tested, only on a default, fresh installation of Ubuntu 26.04.

### 3) Compile spec2parse binary

Have a recent, stable Rust compiler toolchain installed and run:

`cargo build --release --bin spec2parse`

Binary is created in target/release/spec2parse. Copy to somewhere accessible.


### 4) Extract data from specification

The conversion example shown above should, for example, have produced a set of `tpm185-part2.xxxx.fodg` files, where xxxx is a page number starting from 0001.

Call `spec2parse` with `--help` to see all available commands. For example:

Check if the tables are properly detected: \
`spec2parse parse -i fodg/tpm185-part2.0001.fodg --list-tables`

Check on a specific table, for example pretty print table 3: \
`spec2parse parse -i fodg/tpm185-part2.0001.fodg --pretty --table 3`

```
// Extracted from Trusted Platform Module 2.0 Library Part 2: Structures v185 2026/03/12, page 38
[IfTyp] Table 3: Definition of Base Types
Type     | Name   | Description
---------+--------+-------------------------------------
uint8_t  | UINT8  | unsigned, 8-bit integer
uint8_t  | BYTE   | unsigned 8-bit integer
....
```

Finally, create a structured JSON file of all the tables in the specification: \
`spec2parse parse -i fodg/tpm185-part2.0001.fodg -o tpm185-part2.json`

```
{
  "title": "Table 3: Definition of Base Types"
  "number": 3,
  "startpage": 38,
  "columns": [ "Type", "Name", "Description" ],
    "rows": [
      [ "uint8_t", "UINT8", "unsigned, 8-bit integer" ],
      [ "uint8_t", "BYTE",  "unsigned 8-bit integer"  ],
    ...
```


### 5) Generate code fragments from data

Using the structured JSON data output, code fragments can be generated:

`spec2parse generate -i tpm185-part2.json --table 3 ....`

However, in this release, code generation is not yet implemented.



### Declaration of LLM/AI Use

Short snippets of the code in this project have been developed with the assistance of generative AI/LLM. We used the "GLM-4.5-Air" model as provided by z.ai, which is published as "We have open-sourced the base models, hybrid reasoning models, and FP8 versions of the hybrid reasoning models for both GLM-4.5 and GLM-4.5-Air. They are released under the MIT open-source license and can be used commercially and for secondary development" (see https://huggingface.co/zai-org/GLM-4.5-Air).
Any similarity of generated code to existing code is assumed to be coincidental, as happens naturally with short code fragments.


### FAQ

Q: There are numerous text-from-PDF tools, why a custom tool?

A: PDF is a "ready to print" format and does not retain the original source like e.g. a Word document. Reconstruction of the original text structure and formattings from a PDF is non-trivial. Several tools do implement different approaches, which for many documents is "good enough". However, after tries with different tools, the current custom implementation was selected as overall the best approach (but also took some time to implement and is unfortunately forever imperfect...)

Q: Use of novel AI/LLM/OCR/...?

A: In our experience, at this time, LLMs can (still) not be trusted to not subtly corrupt things. Therefore, LLMs are useful for development of the code that extracts and transforms data, but should not modify data directly.
