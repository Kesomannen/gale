import { getVersion } from '@tauri-apps/api/app';
import type { PageLoad } from './$types';

const URL = 'https://raw.githubusercontent.com/Kesomannen/gale/master/CHANGELOG.md';

export const load: PageLoad = async () => {
	let changelog = await fetch(URL).then((res) => res.text());


  // remove Unreleased section
	let unreleasedIndex = changelog.indexOf('## Unreleased');
	let nextVersionIndex = changelog.indexOf('## 0.', unreleasedIndex + 1);

	if (unreleasedIndex !== -1 && nextVersionIndex !== -1) {
		changelog = changelog.slice(0, unreleasedIndex) + changelog.slice(nextVersionIndex);
	}

  let version = await getVersion();

	return {
    changelog,
    version
  };
};
