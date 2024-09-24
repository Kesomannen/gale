<script lang="ts">
	import Markdown from '$lib/components/Markdown.svelte';
	import { getVersion } from '@tauri-apps/api/app';
	import { onMount } from 'svelte';

	const URL = 'https://raw.githubusercontent.com/Kesomannen/gale/master/CHANGELOG.md';

	let version: string;
	let changelog: string;

	onMount(async () => {
		changelog = await fetch(URL).then((res) => res.text());

		// remove Unreleased section
		let unreleasedIndex = changelog.indexOf('## Unreleased');
		let nextVersionIndex = changelog.indexOf('## 0.', unreleasedIndex + 1);

		if (unreleasedIndex !== -1 && nextVersionIndex !== -1) {
			changelog = changelog.slice(0, unreleasedIndex) + changelog.slice(nextVersionIndex);
		}

		version = await getVersion();
	});
</script>

<div class="absolute bottom-1 right-6 text-slate-400">
	Gale v{version}
</div>

<div class="w-full overflow-y-auto px-6 py-2 text-slate-100">
	<Markdown source={changelog} />
</div>
