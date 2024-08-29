<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { MarkdownResponse, Mod } from '../models';
	import { shortenNum, timeSince } from '../util';
	import { Button, DropdownMenu } from 'bits-ui';
	import { slide } from 'svelte/transition';
	import Popup from '$lib/components/Popup.svelte';

	import { open } from '@tauri-apps/plugin-shell';
	import { fetch } from '@tauri-apps/plugin-http';
	import { activeGame } from '$lib/stores';
	import { get } from 'svelte/store';
	import ModInfoPopup from './ModInfoPopup.svelte';
	import ModDetailsDropdownItem from './ModDetailsDropdownItem.svelte';
	import Markdown from '$lib/components/Markdown.svelte';
	import ModCardList from './ModCardList.svelte';
	import { T, t } from '$i18n';

	export let mod: Mod;
	export let onClose: () => void;

	let dependenciesOpen = false;

	let readmeOpen = false;
	let readme: ModInfoPopup;

	let changelogOpen = false;
	let changelog: ModInfoPopup;

	function openCommunityUrl(tail?: string) {
		if (!tail) return;

		let game = get(activeGame);
		if (!game) return;

		open(`https://thunderstore.io/c/${game.id}/p/${tail}/`);
	}

	function openIfDefined(url?: string) {
		if (url) open(url);
	}

	let readmePromise: Promise<string | null>;

	async function extractReadme(response: Response) {
		let res = (await response.json()) as MarkdownResponse;

		if (!res.markdown) return null;

		return res.markdown
			.split('\n')
			.filter((line) => !line.startsWith('# '))
			.join('\n');
	}

	$: {
		let url = `https://thunderstore.io/api/experimental/package/${mod.author}/${mod.name}/${mod.version}/readme/`;
		readmePromise = fetch(url).then(extractReadme);
	}
</script>

<div
	class="flex flex-col px-6 pt-6 pb-4 min-w-80 w-[40%] bg-gray-700 text-white border-l border-gray-600 relative"
>
	<DropdownMenu.Root>
		<DropdownMenu.Trigger class="absolute right-2 mt-0.5 rounded-full hover:bg-slate-600 p-1">
			<Icon class="text-slate-200 text-2xl" icon="mdi:dots-vertical" />
		</DropdownMenu.Trigger>
		<DropdownMenu.Content
			class="flex flex-col bg-gray-700 gap-0.5 shadow-xl p-1 rounded-lg border border-gray-500"
			transition={slide}
			transitionConfig={{ duration: 100 }}
		>
			<slot name="dropdown" />

			{#if mod.websiteUrl && mod.websiteUrl.length > 0}
				<ModDetailsDropdownItem
					icon="mdi:open-in-new"
					label="{get(t)['Open website']}"
					onClick={() => openIfDefined(mod.websiteUrl)}
				/>
			{/if}

			{#if mod.donateUrl}
				<ModDetailsDropdownItem
					icon="mdi:heart"
					label="{get(t)["Donate"]}"
					onClick={() => openIfDefined(mod.donateUrl)}
				/>
			{/if}

			<ModDetailsDropdownItem icon="mdi:close" label="{get(t)["Close"]}" onClick={onClose} />
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<div class="mr-6 flex items-center justify-between">
		<Button.Root
			class="text-slate-100 font-bold text-3xl xl:text-4xl hover:underline truncate"
			on:click={() => openCommunityUrl(mod.author + '/' + mod.name)}>{mod.name}</Button.Root
		>
		{#if mod.version}
			<span class="text-slate-300 font-light text-lg xl:text-xl pl-2 align-middle"
				>{mod.version}</span
			>
		{/if}
	</div>

	{#if mod.author}
		<span class="text-slate-300 text-xl xl:text-2xl">
			{get(t)["By"]}
			<Button.Root class="hover:underline" on:click={() => openCommunityUrl(mod.author)}>
				{mod.author}
			</Button.Root>
		</span>
	{/if}

	<div class="flex gap-1 flex-wrap">
		{#if mod.isDeprecated}
			<div class="flex items-center rounded-lg bg-red-600 text-white px-3 py-1 my-1">
				<Icon class="text-xl mr-1" icon="mdi:error" />
				{get(t)["Deprecated"]}
			</div>
		{/if}

		{#if mod.containsNsfw}
			<div class="flex items-center rounded-lg bg-red-600 text-white px-3 py-1 my-1">
				<Icon class="text-xl mr-1" icon="material-symbols:explicit" />
				{get(t)["Contains NSFW"]}
			</div>
		{/if}
	</div>

	{#if mod.categories}
		<div class="flex gap-1 my-2 flex-wrap text-md">
			{#each mod.categories as category}
				<div class="bg-slate-600 rounded-full px-4 py-1 text-blue-100">
					{category}
				</div>
			{/each}
		</div>
	{/if}

	{#if mod.rating || mod.downloads}
		<div class="my-1 flex items-center gap-2 text-lg">
			<Icon class="text-yellow-400" icon="mdi:star" />
			<span class="text-yellow-400 mr-4">{shortenNum(mod.rating ?? 0)}</span>
			<Icon class="text-green-400" icon="mdi:download" />
			<span class="text-green-400">{shortenNum(mod.downloads ?? 0)}</span>
		</div>
	{/if}

	{#if mod.lastUpdated}
		<div class="text-slate-400 text-lg">
			{T(get(t)["Last updated"], {"time": timeSince(new Date(mod.lastUpdated))})}
		</div>
	{/if}

	<p class="text-slate-300 text-xl flex-shrink overflow-hidden mt-3 xl:hidden">
		{mod.description ?? ''}
	</p>

	{#await readmePromise}
		<div class="items-center justify-center w-full h-full hidden xl:flex">
			<Icon class="text-5xl text-slate-300 animate-spin" icon="mdi:loading" />
		</div>
	{:then readme}
		{#if readme}
			<Markdown source={readme} class="hidden xl:block" />
		{:else}
			<p class="text-slate-300 text-xl flex-shrink overflow-hidden mt-3 hidden xl:block">
				{mod.description ?? ''}
			</p>
		{/if}
	{/await}

	<div class="flex-grow" />

	{#if mod.configFile}
		<div
			class="flex items-center gap-2 text-green-400 hover:text-green-300 text-lg hover:underline my-2"
		>
			<Icon class="text-xl" icon="mdi:file-cog" />
			<a href={'/config?file=' + mod.configFile}>{get(t)["Edit config"]}</a>
		</div>
	{/if}

	{#if mod.type === 'remote'}
		<Button.Root
			class="flex items-center text-white pl-3 pr-1.5 py-1 rounded-md bg-slate-600 hover:bg-slate-500 group"
			on:mouseenter={changelog.fetchMarkdown}
			on:click={() => (changelogOpen = true)}
		>
			<Icon icon="mdi:file-document" class="text-lg mr-2" />
			{get(t)["Changelog"]}
		</Button.Root>

		<Button.Root
			class="flex items-center mt-1 text-white pl-3 pr-1.5 py-1 rounded-md bg-slate-600 hover:bg-slate-500 group"
			on:mouseenter={readme.fetchMarkdown}
			on:click={() => (readmeOpen = true)}
		>
			<Icon icon="mdi:info" class="text-lg mr-2" />
			{get(t)["Details"]}
		</Button.Root>
	{/if}

	{#if mod.dependencies && mod.dependencies.length > 0}
		<Button.Root
			class="flex items-center mt-1 text-white pl-3 pr-1 py-1 rounded-md bg-slate-600 hover:bg-slate-500 group"
			on:click={() => (dependenciesOpen = true)}
		>
			<Icon icon="material-symbols:network-node" class="text-lg mr-2" />
			{get(t)["Dependencies"]}
			<div class="bg-slate-500 group-hover:bg-slate-400 px-3 py-0.5 text-sm rounded-md ml-auto">
				{mod.dependencies.length}
			</div>
		</Button.Root>
	{/if}

	<slot />
</div>

<Popup title="{T(get(t)["Dependencies of"], {"name": mod.name})}" bind:open={dependenciesOpen}>
	{#if mod.dependencies}
		<ModCardList names={mod.dependencies} class="mt-4" />
	{/if}
</Popup>

<ModInfoPopup bind:this={readme} bind:open={readmeOpen} {mod} path="readme" />
<ModInfoPopup
	bind:this={changelog}
	bind:open={changelogOpen}
	{mod}
	useLatest={true}
	path="changelog"
/>
