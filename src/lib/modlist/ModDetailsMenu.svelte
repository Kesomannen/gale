<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { DropdownOption, Mod } from '../models';
	import { getTotalDownloads, open, shortenNum } from '../util';
	import { Button, Collapsible, DropdownMenu } from 'bits-ui';
	import { slide } from 'svelte/transition';
	import { quadOut } from 'svelte/easing';

	const dependenciesShown: number = 15;

	export let mod: Mod;
	export let onClose: () => void;
	export let extraDropdownOptions: DropdownOption[] = [];

	const defaultDropdownOptions: DropdownOption[] = [
		{
			icon: 'mdi:close',
			label: 'Close',
			onClick: onClose
		}
	];

	let dropdownOptions = defaultDropdownOptions;

	$: {
		let options = [...defaultDropdownOptions];

		if (extraDropdownOptions.length > 0) {
			options.splice(2, 0, ...extraDropdownOptions);
		}

		if (mod.version.websiteUrl.length > 0) {
			options.splice(2, 0, {
				icon: 'mdi:open-in-new',
				label: 'Open website',
				onClick: () => open(mod.version.websiteUrl)
			});
		}

		if (mod.package.donationLink) {
			options.splice(2, 0, {
				icon: 'mdi:heart',
				label: 'Donate',
				onClick: () => open(mod.package.donationLink!)
			});
		}

		dropdownOptions = options;
	}
</script>

<div
	class="flex flex-col px-6 pb-4 pt-6 min-w-80 w-[40%] bg-gray-700 text-white border-l border-gray-600 relative"
>
	<DropdownMenu.Root>
		<DropdownMenu.Trigger class="absolute right-2 top-18 rounded-full hover:bg-slate-600 p-1">
			<Icon class="text-slate-200 text-2xl" icon="mdi:dots-vertical" />
		</DropdownMenu.Trigger>
		<DropdownMenu.Content
			class="flex flex-col bg-gray-700 gap-0.5 shadow-xl p-2 rounded-lg border border-gray-500"
			transition={slide}
			transitionConfig={{ duration: 100 }}
		>
			{#each dropdownOptions as option}
				<DropdownMenu.Item
					class="flex items-center pl-3 pr-5 py-1 truncate text-slate-300 hover:text-slate-100 text-left rounded-md hover:bg-gray-600 cursor-default"
					on:click={option.onClick}
				>
					{#if option.icon}
						<Icon class="text-xl mr-1" icon={option.icon} />
					{/if}
					{option.label}
				</DropdownMenu.Item>
			{/each}
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<div class="truncate mr-8">
		<Button.Root
			class="text-slate-200 font-semibold text-2xl align-middle hover:underline"
			on:click={() =>
				open(
					`https://thunderstore.io/c/lethal-company/p/${mod.package.owner}/${mod.package.name}/`
				)}>{mod.version.name}</Button.Root
		>
		<span class="text-slate-300 font-light text-lg pl-2 align-middle"
			>{mod.version.versionNumber}</span
		>
	</div>

	<span class="text-slate-400 text-lg">
		By
		<Button.Root
			class="hover:underline"
			on:click={() => open('https://thunderstore.io/c/lethal-company/p/' + mod.package.owner)}
		>
			{mod.package.owner}
		</Button.Root>
	</span>

	<div class="flex gap-2 mt-3 flex-wrap">
		{#each mod.package.categories as category}
			<div class="bg-slate-600 rounded-full px-4 py-1 text-blue-100 text-md">
				{category}
			</div>
		{/each}
	</div>

	<div class="my-4">
		<div class="inline-flex items-center gap-2 mr-6">
			<Icon class="text-yellow-400 text-lg" icon="mdi:star" />
			<span class="text-yellow-400 text-md">{shortenNum(mod.package.ratingScore)}</span>
		</div>
		<div class="inline-flex items-center gap-2">
			<Icon class="text-green-400 text-lg" icon="mdi:download" />
			<span class="text-green-400 text-md">{shortenNum(getTotalDownloads(mod.package))}</span>
		</div>
	</div>

	<p class="text-slate-300 text-lg">{mod.version.description}</p>

	<div class="mt-auto" />

	{#if mod.version.dependencies.length > 0}
		<Collapsible.Root class="mb-2">
			<Collapsible.Trigger
				class="flex items-center w-full text-slate-100 text-xl font-medium px-3 py-1.5 rounded-lg hover:bg-gray-600 group"
			>
				Dependencies
				<div class="bg-gray-600 px-4 py-1 text-sm rounded-md group-hover:bg-gray-500 ml-auto">
					{mod.version.dependencies.length}
				</div>
			</Collapsible.Trigger>
			<Collapsible.Content
				class="pb-4 px-3 mt-1"
				transition={slide}
				transitionConfig={{ duration: 200, easing: quadOut }}
			>
				{#each mod.version.dependencies.slice(0, dependenciesShown) as dependency}
					<p class="text-slate-300 text-sm">{dependency}</p>
				{/each}
				{#if mod.version.dependencies.length > dependenciesShown}
					<p class="text-slate-100 pt-2">
						plus {mod.version.dependencies.length - dependenciesShown} more...
					</p>
				{/if}
			</Collapsible.Content>
		</Collapsible.Root>
	{/if}

	<slot />
</div>
