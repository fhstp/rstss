#!/bin/sh

# Example script to convert the specifications
# adjust paths as needed for local paths

SCRIPTNAME=../convertpdf2fodg.sh

$SCRIPTNAME ../Trusted-Platform-Module-2.0-Library-Part-0-Introduction_Version-185_pub.pdf tpm185-part0
$SCRIPTNAME ../Trusted-Platform-Module-2.0-Library-Part-1-Architecture_Version-185_pub.pdf tpm185-part1
$SCRIPTNAME ../Trusted-Platform-Module-2.0-Library-Part-2-Structures_Version-185_pub.pdf   tpm185-part2
$SCRIPTNAME ../Trusted-Platform-Module-2.0-Library-Part-3-Commands_Version-185_pub.pdf     tpm185-part3

$SCRIPTNAME ../TCG-Algorithm-Registry-Version-2.0_pub.pdf                                             algo
$SCRIPTNAME ../Registry-of-Reserved-TPM-2.0-Handles-and-Localities-Version-1.2-Revision-1.00_pub.pdf  registry
$SCRIPTNAME ../TCG-TPM-Vendor-ID-Registry-Family-1.2-and-2.0-Version-1.07-Revision-0.02_pub.pdf       vendor
