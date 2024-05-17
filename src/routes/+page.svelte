<script lang="ts">
	import Markdown from '$lib/Markdown.svelte';
	import { getVersion } from '@tauri-apps/api/app';
	import { onMount } from 'svelte';

	let version: string;
	let changelogPromise: Promise<string>;

	onMount(async () => {
		getVersion().then((v) => {
			version = v;
		});

		let response = await fetch(
			'https://raw.githubusercontent.com/Kesomannen/gale/master/CHANGELOG.md'
		);
		changelogPromise = response.text();
	});
</script>

<div class="absolute right-6 bottom-1 text-slate-400">
	Gale v{version}
</div>

<div class="px-6 overflow-y-auto text-slate-100 w-full">
	{#await changelogPromise}
		Loading changelog...
	{:then changelog}
		<Markdown source={changelog} />
	{:catch error}
		Failed to load changelog: {error.message}
	{/await}
</div>