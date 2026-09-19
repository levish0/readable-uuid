import { writeFileSync } from 'node:fs';
const packages = ['human-id', 'unique-names-generator', 'humanhash', 'wordhash'];
const results = await Promise.all(
  packages.map(async (name) => {
    const url = `https://api.npmjs.org/downloads/point/last-week/${name}`;
    try {
      const response = await fetch(url, { signal: AbortSignal.timeout(15000) });
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      return { source: url, ...(await response.json()) };
    } catch (error) {
      return { package: name, source: url, error: String(error) };
    }
  }),
);
const data = { fetchedAt: new Date().toISOString(), packages: results };
writeFileSync(new URL('./popularity.json', import.meta.url), JSON.stringify(data, null, 2) + '\n');
console.table(
  results.map(({ package: name, downloads, start, end, error }) => ({
    name,
    downloads,
    start,
    end,
    error,
  })),
);
