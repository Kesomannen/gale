<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import Markdown from '$lib/components/ui/Markdown.svelte';

	import ModInfoDialog from '../dialogs/ModInfoDialog.svelte';
	import ModCardList from '../ui/ModCardList.svelte';
	import ModContextMenuContent from './ModContextMenuContent.svelte';

	import { ModType, type Mod, type ModContextItem } from '$lib/types';
	import {
		communityUrl,
		formatModName,
		getMarkdown,
		modIconSrc,
		shortenFileSize,
		shortenNum,
		timeSince
	} from '$lib/util';

	import { DropdownMenu } from 'bits-ui';

	import Icon from '@iconify/svelte';
	import { type Snippet } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import config from '$lib/state/config.svelte';
	import { goto } from '$app/navigation';
	import InfoBox from '../ui/InfoBox.svelte';
	import translation from '$lib/state/translation.svelte';

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
	let readme: ModInfoDialog;

	let changelogOpen = $state(false);
	let changelog: ModInfoDialog;

	let allContextItems = $derived([
		...contextItems,
		{
			label: m.modDetails_allContextItems_close(),
			icon: 'mdi:close',
			onclick: onclose
		}
	]);

	let readmePromise: Promise<string | null> | null = $state(null);

	let displayName = $derived(translation.getDisplayName(mod.uuid, formatModName(mod.name)));
	let displayDescription = $derived(translation.getDisplayDescription(mod.uuid, mod.description));
	let isTranslatingThis = $derived(translation.isTranslating(mod.uuid));
	let hasTranslation = $derived(!!translation.getTranslation(mod.uuid));

	function formatReadme(readme: string | null) {
		if (readme === null) return null;

		return readme
			.split('\n')
			.filter((line) => !line.startsWith('# '))
			.join('\n');
	}

	$effect(() => {
		readmePromise = getMarkdown(mod, 'readme').then(formatReadme);
	});

	async function handleTranslate() {
		if (hasTranslation || isTranslatingThis) return;
		await translation.translateMod(mod);
	}
</script>

<div class="relative flex w-[40%] min-w-72 flex-col p-4 text-white">
	<DropdownMenu.Root>
		<DropdownMenu.Trigger
			class="bg-primary-800 hover:bg-primary-700 absolute right-4 mt-1 rounded-full p-1"
		>
			<Icon class="text-primary-200 text-3xl" icon="mdi:dots-vertical" />
		</DropdownMenu.Trigger>
		<ModContextMenuContent {mod} {locked} items={allContextItems} type="dropdown" />
	</DropdownMenu.Root>

	<div class="-mr-3 grow overflow-x-hidden overflow-y-auto pr-3 pb-2">
		<div class="mb-4 flex flex-wrap gap-4 xl:items-center">
			<img src={modIconSrc(mod)} class="max-h-30 max-w-30 rounded-lg" alt="" />

			<div>
				<svelte:element
					this={mod.type === ModType.Remote ? 'a' : 'div'}
					class={[
						'pr-4 text-left text-3xl font-bold wrap-break-word text-white xl:text-4xl',
						mod.type === ModType.Remote && 'hover:underline'
					]}
					href={communityUrl(`${mod.author}/${mod.name}`)}
					target="_blank">{displayName}</svelte:element
				>

				{#if mod.author}
					<a
						class="text-primary-400 hover:text-primary-300 block text-lg hover:underline xl:text-xl"
						href={communityUrl(mod.author)}
						target="_blank"
					>
						{mod.author}
					</a>
				{/if}

				{#if mod.version}
					<div class="text-primary-400 text-lg xl:text-xl">{mod.version}</div>
				{/if}
			</div>
		</div>

		{#if mod.isDeprecated}
			<InfoBox type="warning">
				{m.modDetails_deprecated()}
			</InfoBox>
		{/if}

		{#if mod.containsNsfw}
			<InfoBox type="warning">
				{m.modDetails_NSFW()}
			</InfoBox>
		{/if}

		{#if mod.categories}
			<div class="mt-2 mb-1 flex flex-wrap gap-1">
				{#each mod.categories as category}
					<div class="bg-primary-700 text-primary-200 rounded-full px-4 py-1">
						{category}
					</div>
				{/each}
			</div>
		{/if}

		<div class="mt-2 flex items-center gap-1.5 text-lg">
			{#if mod.rating !== null}
				<Icon class="shrink-0 text-yellow-500" icon="mdi:star" />
				<span class="mr-4 text-yellow-500">{shortenNum(mod.rating)}</span>
			{/if}
			{#if mod.downloads !== null}
				<Icon class="shrink-0 text-green-500" icon="mdi:download" />
				<span class="mr-4 text-green-500">{shortenNum(mod.downloads)}</span>
			{/if}
			<Icon class="text-primary-400 shrink-0" icon="mdi:weight" />
			<span class="text-primary-400">{shortenFileSize(mod.fileSize)}</span>
		</div>

		{#if mod.lastUpdated !== null}
			<div class="text-primary-400 mt-1 text-lg">
				{m.modDetails_lastUpdated({ time: timeSince(new Date(mod.lastUpdated)) })}
			</div>
		{/if}

		{#if displayDescription !== null}
			<p class="text-primary-300 mt-2 text-xl lg:hidden">
				{displayDescription}
			</p>
		{/if}

		<div class="hidden lg:block">
			{#await readmePromise}
				<div role="status" class="animate-pulse">
					<div class="bg-primary-700 mt-4 h-8 w-80 rounded-xl"></div>
					<div class="bg-primary-700 mt-6 h-3 max-w-125 rounded-full"></div>
					<div class="bg-primary-700 mt-2.5 h-3 max-w-115 rounded-full"></div>
					<div class="bg-primary-700 mt-2.5 mb-4 h-3 max-w-100 rounded-full"></div>
				</div>
			{:then readme}
				<Markdown source={readme ?? m.modDetails_noFound()} />
			{/await}
		</div>
	</div>

	{#if mod.configFile}
		<div
			class="text-accent-400 hover:text-accent-300 my-2 flex items-center gap-2 text-lg hover:underline"
		>
			<Icon class="text-xl" icon="mdi:file-cog" />
			<button
				onclick={() => {
					const file = config.findFileByPath(mod.configFile!);
					if (!file) {
						console.error('Config file not found for mod', mod.configFile);
						return;
					}

					config.selectedFile = file;
					goto('/config');
				}}>{m.modDetails_editConfig()}</button
			>
		</div>
	{/if}

	{#snippet button(icon: string, label: string, onclick: () => void, onmouseenter?: () => void)}
		<button
			class="group bg-primary-700 hover:bg-primary-600 text-primary-200 mb-1 flex items-center rounded-md py-1 pr-1.5 pl-3"
			{onmouseenter}
			{onclick}
		>
			<Icon {icon} class={["mr-2 text-lg", isTranslatingThis && "animate-spin"]} />
			{label}
		</button>
	{/snippet}

	{@render button(
		'mdi:file-document',
		m.modDetails_changeLog(),
		() => (changelogOpen = true),
		() => changelog.fetchMarkdown()
	)}
	{@render button(
		'mdi:info',
		m.modDetails_details(),
		() => (readmeOpen = true),
		() => readme.fetchMarkdown()
	)}

	{#if mod.dependencies !== null && mod.dependencies.length > 0}
		{@render button(
			'material-symbols:network-node',
			`${m.modDetails_dependencies()} (${mod.dependencies.length})`,
			() => (dependenciesOpen = true)
		)}
	{/if}

	{#if translation.prefs?.enabled && translation.prefs?.apiUrl && translation.prefs?.apiKey}
		{@render button(
			isTranslatingThis ? 'mdi:loading' : hasTranslation ? 'mdi:check-circle' : 'mdi:translate',
			isTranslatingThis ? 'Translating...' : hasTranslation ? 'Translated' : 'Translate',
			handleTranslate
		)}
	{/if}

	{@render children?.()}
</div>

<Dialog title="Dependencies of {mod.name}" bind:open={dependenciesOpen}>
	{#if mod.dependencies}
		<ModCardList mods={mod.dependencies.map((fullName) => ({ fullName }))} class="mt-4" />
	{/if}
</Dialog>

<ModInfoDialog bind:this={readme} bind:open={readmeOpen} {mod} type="readme" />
<ModInfoDialog
	bind:this={changelog}
	bind:open={changelogOpen}
	{mod}
	useLatest={true}
	type="changelog"
/>
