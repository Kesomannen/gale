<script lang="ts">
	import Markdown from '$lib/components/Markdown.svelte';
	import { getVersion } from '@tauri-apps/api/app';
	import { onMount } from 'svelte';

	const URL = 'https://raw.githubusercontent.com/Kesomannen/gale/master/CHANGELOG.md'

	let version: string;
	let changelogPromise: Promise<string>;

	onMount(async () => {
		getVersion().then((v) => {
			version = v;
		});

		let response = await fetch(URL);
		changelogPromise = response.text()
			.then((text) => {
				let unreleasedIndex = text.indexOf('## Unreleased');
				let nextVersionIndex = text.indexOf('## 0.', unreleasedIndex + 1);

				if (unreleasedIndex !== -1 && nextVersionIndex !== -1) {
					text = text.slice(0, unreleasedIndex) + text.slice(nextVersionIndex);
				}

				return text;
			})
	});
</script>

<div class="absolute right-6 bottom-1 text-slate-400">
	Gale v{version}
</div>

<div class="px-6 py-2 overflow-y-auto text-slate-100 w-full">
	{#await changelogPromise}
		Loading changelog...
	{:then changelog}
		<Markdown source={changelog} />
	{:catch error}
		Failed to load changelog: {error.message}
	{/await}
</div>