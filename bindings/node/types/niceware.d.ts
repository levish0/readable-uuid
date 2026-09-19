declare module 'niceware' {
  export function bytesToPassphrase(bytes: Buffer): string[];
  export function passphraseToBytes(words: string[]): Buffer;
}
