export interface Options {
  separator?: string;
}

export function encode(uuid: string, options?: Options): string;
export function decode(phrase: string, options?: Options): string;
export function encodeBatch(uuids: readonly string[], options?: Options): string[];
export function decodeBatch(phrases: readonly string[], options?: Options): string[];
