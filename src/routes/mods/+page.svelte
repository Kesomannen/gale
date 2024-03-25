<script lang="ts">
	import { invoke } from '@tauri-apps/api';

	import ModListItem from '$lib/modlist/ModListItem.svelte';
	import ModDetailsMenu from '$lib/modlist/ModDetailsMenu.svelte';
	import InstallProgressPopup from '$lib/modlist/InstallProgressPopup.svelte';

	import type { Mod } from '$lib/models';
	import { shortenFileSize } from '$lib/util';
	import { inProfile } from '$lib/profile';

	import Icon from '@iconify/svelte';

	import { Button, Popover, Select, Separator } from 'bits-ui';
	import { blur, fade, slide } from 'svelte/transition';
	import { get } from 'svelte/store';
	import ModListSortOption from '$lib/modlist/ModListSortOption.svelte';
	import { quadOut } from 'svelte/easing';
	import { invokeCommand } from '$lib/error';

	const pageSize = 10;
	const sortOptions = [
		{ value: undefined, label: 'Relevance' },
		{ value: 'Rating', label: 'Rating' },
		{ value: 'Downloads', label: 'Downloads' },
		{ value: 'LastUpdated', label: 'Last updated' }
	];

	const allCategories = [
		'Mods',
		'Modpacks',
		'Interiors',
		'Audio',
		'Tools',
		'Moons',
		'Client-side',
		'Emotes',
		'Cosmetics',
		'TV Videos'
	];

	let page = 0;

	let searchTerm: string | undefined;
	let categories: string[] = [];
	let includeNsfw = false;
	let includeDeprecated = false;
	let sortBy: { value: string | undefined; label: string } = sortOptions[0];

	let mods: Mod[] | undefined;
	let activeMod: Mod | undefined;
	let activeDownloadSize: number | undefined;

	$: {
		if (!$inProfile && activeMod !== undefined) {
			invoke<number>('get_download_size', {
				packageUuid: activeMod?.package.uuid4
			}).then((size) => (activeDownloadSize = size));
		}
	}

	$: {
		const command = $inProfile ? 'query_mods_in_profile' : 'query_all_mods';
		invoke<Mod[]>(command, {
			args: {
				page: page,
				page_size: pageSize,
				search_term: searchTerm,
				categories,
				include_nsfw: includeNsfw,
				include_deprecated: includeDeprecated,
				descending: true,
				sort_by: sortBy.value
			}
		}).then((result) => (mods = result));
	}

	let installingMod: Mod | undefined;
	let isInstalling = false;

	async function onModAction(mod: Mod | undefined) {
		if (mod === undefined) return;

		if (get(inProfile)) {
			invokeCommand('remove_mod', { packageUuid: mod.package.uuid4 });
		} else {
			installingMod = mod;
			isInstalling = true;
			try {
				await invokeCommand('install_mod', { packageUuid: mod.package.uuid4 });
				await new Promise((r) => setTimeout(r, 1000));
			} finally {
				isInstalling = false;
				installingMod = undefined;
			}
		}
	}

	function onModClicked(mod: Mod) {
		if (activeMod === undefined || activeMod.package.uuid4 !== mod.package.uuid4) {
			activeMod = mod;
		} else {
			activeMod = undefined;
		}
	}

	function switchMode() {
		searchTerm = undefined;
		page = 0;
		activeMod = undefined;
		inProfile.set(!get(inProfile));
	}

	function onFilterChange(category: string, value: boolean) {
		if (value) {
			categories = [...categories, category];
		} else {
			categories = categories.filter((c) => c !== category);
		}
	}
</script>

<div class="flex flex-grow">
	<div class="flex flex-col flex-grow w-[60%] px-4 py-4">
		<div class="flex text-slate-200 text-xl font-medium pl-2">
			<Icon
				class="text-slate-300 text-2xl mr-2"
				icon={$inProfile ? 'mdi:account-circle' : 'material-symbols:browse'}
			/>
			{$inProfile ? 'Profile' : 'Mod list'}
			<Button.Root
				class="px-4 font-normal text-sm ml-auto rounded-lg bg-gray-700 hover:bg-gray-600"
				on:click={switchMode}
			>
				{$inProfile ? 'Switch to mod list' : 'Switch to profile'}
			</Button.Root>
		</div>

		<div class="flex gap-2 my-2">
			<div class="relative flex-grow">
				<input
					type="text"
					class="w-full py-2 pr-10 pl-12 rounded-lg bg-gray-900 text-slate-300 truncate"
					bind:value={searchTerm}
					placeholder="Search for mods..."
				/>
				<Icon class="absolute left-[12px] top-[9px] text-slate-400 text-2xl" icon="mdi:magnify" />
			</div>

			<Select.Root items={sortOptions} bind:selected={sortBy}>
				<Select.Trigger
					class="flex items-center flex-shrink-0 w-48 bg-gray-900 rounded-lg px-3 
                                       border border-gray-500 border-opacity-0 hover:border-opacity-100"
				>
					<Icon class="text-slate-400 text-2xl mr-2" icon="mdi:sort" />
					<Select.Value class="text-slate-300 text-left w-full" />
					<Icon class="text-slate-400 text-2xl ml-auto" icon="mdi:chevron-down" />
				</Select.Trigger>
				<Select.Content
					class="flex flex-col bg-gray-800 gap-0.5 shadow-xl p-2 w-48 rounded-lg border border-gray-600"
					transition={slide}
					transitionConfig={{ duration: 100 }}
				>
					{#each sortOptions as item}
						<Select.Item
							value={item.value}
							class="flex items-center px-3 py-1 text-slate-400 hover:text-slate-200 text-left rounded-lg hover:bg-gray-700 cursor-default"
						>
							{item.label}
						</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
		</div>

		<div class="flex gap-2 mb-2">
			<div class="flex items-center flex-shrink-0">
				<Button.Root
					class="p-1 hover:bg-gray-700 rounded-lg transition-all group"
					on:click={() => page--}
					disabled={page === 0}
				>
					<Icon
						class="text-white text-2xl group-disabled:text-slate-400 align-middle"
						icon="mdi:chevron-left"
					/>
				</Button.Root>

				<div class="text-md px-4 text-slate-200">Page {page + 1}</div>

				<Button.Root class="p-1 hover:bg-gray-700 rounded-lg" on:click={() => page++}>
					<Icon class="text-white text-2xl align-middle" icon="mdi:chevron-right" />
				</Button.Root>
			</div>

			<Popover.Root>
				<Popover.Trigger
					class="flex items-center bg-gray-900 rounded-lg px-3 py-1.5
                                        border border-gray-500 border-opacity-0 hover:border-opacity-100 truncate"
				>
					<Icon class="text-slate-400 mr-2 text-xl flex-shrink-0" icon="mdi:filter" />
					<div class="text-slate-300 text-left mr-2">Filter</div>
					{#if categories.length > 0}
						<div class="text-slate-500 truncate italic">
							{categories.join(', ')}
						</div>
					{/if}
					<Icon class="text-slate-400 ml-2 text-xl flex-shrink-0" icon="mdi:chevron-down" />
				</Popover.Trigger>
				<Popover.Content
					class="flex flex-col bg-gray-800 gap-2 shadow-xl py-4 pl-4 pr-8 rounded-lg border border-gray-600"
					transition={slide}
					transitionConfig={{ duration: 150, easing: quadOut }}
				>
					<ModListSortOption label="Include NSFW" bind:value={includeNsfw} />
					<ModListSortOption label="Include deprecated" bind:value={includeDeprecated} />
					<Separator.Root class="w-full h-[1px] bg-gray-600 my-1" />
					<div class="grid gap-y-2 gap-x-4 grid-cols-3">
						{#each allCategories as category}
							<ModListSortOption
								label={category}
								value={categories.includes(category)}
								onChange={(value) => onFilterChange(category, value)}
							/>
						{/each}
					</div>
				</Popover.Content>
			</Popover.Root>
		</div>

		{#if mods}
			{#if mods.length === 0}
				<p class="text-white text-lg pl-2">No matches found 😥</p>
			{:else}
				{#each mods as mod}
					<ModListItem onClick={onModClicked} {mod}>
						{#if !$inProfile}
							<Button.Root
								class="flex items-center justify-center gap-2 rounded-lg text-lg font-medium text-slate-100
																					bg-green-600 hover:bg-green-500 p-3 mr-1 ml-3"
								on:click={() => onModAction(mod)}
							>
								<Icon icon="mdi:download" class="text-white text-xl align-middle" />
							</Button.Root>
						{/if}
					</ModListItem>
				{/each}
			{/if}
		{:else}
			<p class="text-white text-lg pl-2">Loading mods... 🥱</p>
		{/if}
	</div>

	{#if activeMod}
		<ModDetailsMenu mod={activeMod} onClose={() => (activeMod = undefined)}>
			{#if !$inProfile}
				<Button.Root
					class="flex items-center justify-center gap-2 rounded-lg text-lg font-medium text-slate-100
                        bg-green-600 hover:bg-green-500 py-2"
					on:click={() => onModAction(activeMod)}
				>
					<Icon icon="mdi:download" class="text-white text-xl align-middle" />
					Install
					{#if activeDownloadSize !== undefined && activeDownloadSize > 0}
						({shortenFileSize(activeDownloadSize ?? 0)})
					{/if}
				</Button.Root>
			{/if}
		</ModDetailsMenu>
	{/if}

	<InstallProgressPopup
		modName={installingMod?.version.name ?? 'Unknown'}
		bind:open={isInstalling}
	/>
</div>
