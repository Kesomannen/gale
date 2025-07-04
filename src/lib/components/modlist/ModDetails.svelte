<script lang="ts">
	import Popup from '$lib/components/ui/Popup.svelte';
	import Markdown from '$lib/components/ui/Markdown.svelte';

	import ModInfoPopup from './ModInfoPopup.svelte';
	import ModCardList from './ModCardList.svelte';
	import ModContextMenuContent from './ModContextMenuContent.svelte';

	import { ModType, type Mod, type ModContextItem } from '$lib/types';
	import {
		communityUrl,
		shortenFileSize,
		shortenNum,
		thunderstoreIconUrl,
		timeSince
	} from '$lib/util';

	import { DropdownMenu } from 'bits-ui';

	import Icon from '@iconify/svelte';
	import { type Snippet } from 'svelte';
	import * as api from '$lib/api';

	type Props = {
		mod: Mod;
		contextItems?: ModContextItem[];
		locked: boolean;
		onclose: () => void;
		children?: Snippet;
	};

	let { mod, contextItems = [], locked, onclose, children }: Props = $props();

	let dependenciesOpen = $state(false);

	let readmeOpen = $state(false);
	let readme: ModInfoPopup;

	let changelogOpen = $state(false);
	let changelog: ModInfoPopup;

	let allContextItems = $derived([
		...contextItems,
		{
			label: 'Close',
			icon: 'mdi:close',
			onclick: onclose
		}
	]);

	let readmePromise: Promise<string | null> | null = $state(null);

	function formatReadme(readme: string | null) {
		if (readme === null) return null;

		return readme
			.split('\n')
			.filter((line) => !line.startsWith('# '))
			.join('\n');
	}

	$effect(() => {
		if (mod.type === ModType.Remote) {
			readmePromise = api.thunderstore
				.getMarkdown(
					{
						packageUuid: mod.uuid,
						versionUuid: mod.versionUuid
					},
					'readme'
				)
				.then(formatReadme);
		}
	});
</script>

<div
	class="border-primary-600 bg-primary-700 relative flex w-[40%] min-w-72 flex-col border-l px-6 pt-6 pb-4 text-white"
>
	<DropdownMenu.Root>
		<DropdownMenu.Trigger
			class="bg-primary-700 hover:bg-primary-600 absolute right-2 mt-0.5 rounded-full p-1"
		>
			<Icon class="text-primary-200 text-2xl" icon="mdi:dots-vertical" />
		</DropdownMenu.Trigger>
		<ModContextMenuContent style="light" {mod} {locked} items={allContextItems} type="dropdown" />
	</DropdownMenu.Root>

	<div class="light-scrollbar grow overflow-x-hidden overflow-y-auto pb-2">
		<div class="flex flex-wrap gap-4 xl:items-center">
			<img
				src={thunderstoreIconUrl(`${mod.author}-${mod.name}-${mod.version}`)}
				class="max-h-30 max-w-30 rounded-lg"
				alt=""
			/>

			<div>
				<a
					class="pr-4 text-left text-3xl font-bold break-words text-white hover:underline xl:text-4xl"
					href={communityUrl(`${mod.author}/${mod.name}`)}
					target="_blank">{mod.name.replace(/_/g, ' ')}</a
				>

				{#if mod.author}
					<div class="text-primary-300 text-xl xl:text-2xl">
						by
						<a class="hover:underline" href={communityUrl(mod.author)} target="_blank">
							{mod.author}
						</a>
					</div>
				{/if}

				{#if mod.version}
					<div class="text-primary-300 text-xl xl:text-2xl">v{mod.version}</div>
				{/if}
			</div>
		</div>

		<div class="flex flex-wrap gap-1">
			{#if mod.isDeprecated}
				<div class="my-1 flex items-center rounded-lg bg-red-600 px-3 py-1 text-white">
					<Icon class="mr-1 text-xl" icon="mdi:error" />
					Deprecated
				</div>
			{/if}

			{#if mod.containsNsfw}
				<div class="my-1 flex items-center rounded-lg bg-red-600 px-3 py-1 text-white">
					<Icon class="mr-1 text-xl" icon="material-symbols:explicit" />
					Contains NSFW
				</div>
			{/if}
		</div>

		{#if mod.categories}
			<div class="mt-4 mb-1 flex flex-wrap gap-1">
				{#each mod.categories as category}
					<div class="bg-primary-600 text-primary-200 rounded-full px-4 py-1">
						{category}
					</div>
				{/each}
			</div>
		{/if}

		<div class="mt-2 flex items-center gap-1.5 text-lg">
			{#if mod.rating !== null}
				<Icon class="shrink-0 text-yellow-400" icon="mdi:star" />
				<span class="mr-4 text-yellow-400">{shortenNum(mod.rating)}</span>
			{/if}
			{#if mod.downloads !== null}
				<Icon class="shrink-0 text-green-400" icon="mdi:download" />
				<span class="mr-4 text-green-400">{shortenNum(mod.downloads)}</span>
			{/if}
			<Icon class="text-primary-400 shrink-0" icon="mdi:weight" />
			<span class="text-primary-400">{shortenFileSize(mod.fileSize)}</span>
		</div>

		{#if mod.lastUpdated !== null}
			<div class="text-primary-400 mt-1 text-lg">
				Last updated {timeSince(new Date(mod.lastUpdated))} ago
			</div>
		{/if}

		{#if mod.description !== null}
			<p class="text-primary-300 mt-2 text-xl lg:hidden">
				{mod.description}
			</p>
		{/if}

		<div class="hidden lg:block">
			{#await readmePromise}
				<div role="status" class="animate-pulse">
					<div class="bg-primary-600 mt-4 h-8 w-80 rounded-xl"></div>
					<div class="bg-primary-600 mt-6 h-3 max-w-[500px] rounded-full"></div>
					<div class="bg-primary-600 mt-2.5 h-3 max-w-[460px] rounded-full"></div>
					<div class="bg-primary-600 mt-2.5 mb-4 h-3 max-w-[400px] rounded-full"></div>
				</div>
			{:then readme}
				<Markdown source={readme ?? 'No readme found'} />
			{/await}
		</div>
	</div>

	{#if mod.configFile}
		<div
			class="text-accent-400 hover:text-accent-300 my-2 flex items-center gap-2 text-lg hover:underline"
		>
			<Icon class="text-xl" icon="mdi:file-cog" />
			<a href={'/config?file=' + mod.configFile}>Edit config</a>
		</div>
	{/if}

	{#if mod.type === 'remote'}
		<button
			class="group bg-primary-600 hover:bg-primary-500 flex items-center rounded-md py-1 pr-1.5 pl-3 text-white"
			onmouseenter={() => changelog.fetchMarkdown()}
			onclick={() => (changelogOpen = true)}
		>
			<Icon icon="mdi:file-document" class="mr-2 text-lg" />
			Changelog
		</button>

		<button
			class="group bg-primary-600 hover:bg-primary-500 mt-1 flex items-center rounded-md py-1 pr-1.5 pl-3 text-white"
			onmouseenter={() => readme.fetchMarkdown()}
			onclick={() => (readmeOpen = true)}
		>
			<Icon icon="mdi:info" class="mr-2 text-lg" />
			Details
		</button>
	{/if}

	{#if mod.dependencies !== null && mod.dependencies.length > 0}
		<button
			class="group bg-primary-600 hover:bg-primary-500 mt-1 flex items-center rounded-md py-1 pr-1 pl-3 text-white"
			onclick={() => (dependenciesOpen = true)}
		>
			<Icon icon="material-symbols:network-node" class="mr-2 text-lg" />
			Dependencies
			<div class="bg-primary-500 group-hover:bg-primary-400 ml-auto rounded-md px-3 py-0.5 text-sm">
				{mod.dependencies.length}
			</div>
		</button>
	{/if}

	{@render children?.()}
</div>

<Popup
	large={(mod.dependencies?.length ?? 0) > 10}
	title="Dependencies of {mod.name}"
	bind:open={dependenciesOpen}
>
	{#if mod.dependencies}
		<ModCardList names={mod.dependencies} class="mt-4" />
	{/if}
</Popup>

<ModInfoPopup bind:this={readme} bind:open={readmeOpen} {mod} kind="readme" />
<ModInfoPopup
	bind:this={changelog}
	bind:open={changelogOpen}
	{mod}
	useLatest={true}
	kind="changelog"
/>
