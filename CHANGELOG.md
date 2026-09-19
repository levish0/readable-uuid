# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Encode every UUID losslessly as eight codewords and decode it back.
- Encode with one fixed, reviewed 65,536-word codebook.
- Provide Rust and WebAssembly single and batch APIs for Node.js and browsers.
- Compare both directions with Niceware and BIP39.
