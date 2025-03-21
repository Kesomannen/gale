<script lang="ts">
	import InputField from '$lib/components/InputField.svelte';
	import FormField from '$lib/components/FormField.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import BigButton from '$lib/components/BigButton.svelte';
	import PathField from '$lib/components/PathField.svelte';
	import Markdown from '$lib/components/Markdown.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import Link from '$lib/components/Link.svelte';
	import ApiKeyPopup, { apiKeyPopupOpen } from '$lib/prefs/ApiKeyPopup.svelte';

	import { invokeCommand } from '$lib/invoke';
	import type { ModpackArgs, PackageCategory } from '$lib/models';
	import { activeProfile, activeGame, categories } from '$lib/stores';
	import { open } from '@tauri-apps/plugin-dialog';
	import { onDestroy } from 'svelte';
	import { fade } from 'svelte/transition';
	import Icon from '@iconify/svelte';

	import { Button, Dialog, Select } from 'bits-ui';
	import Popup from '$lib/components/Popup.svelte';
	import Checklist from '$lib/components/Checklist.svelte';
	import ResizableInputField from '$lib/components/ResizableInputField.svelte';

	const URL_PATTERN =
		'[Hh][Tt][Tt][Pp][Ss]?://(?:(?:[a-zA-Z\u00a1-\uffff0-9]+-?)*[a-zA-Z\u00a1-\uffff0-9]+)(?:.(?:[a-zA-Z\u00a1-\uffff0-9]+-?)*[a-zA-Z\u00a1-\uffff0-9]+)*(?:.(?:[a-zA-Z\u00a1-\uffff]{2,}))(?::d{2,5})?(?:/[^s]*)?';

	let name: string;
	let author: string;
	let selectedCategories: PackageCategory[] = [];
	let nsfw: boolean;
	let description: string;
	let readme: string;
	let changelog: string;
	let versionNumber: string;
	let iconPath: string;
	let websiteUrl: string;
	let includeDisabled: boolean;
	let includeFiles = new Map<string, boolean>();

	let donePopupOpen = false;
	let loading: string | null = null;

	let includedFileCount = 0;

	$: {
		$activeProfile;
		refresh();
	}

	// some communities don't have a specific modpack category
	$: modpackCategoryExists = $categories.some((category) => category.slug === 'modpacks');

	// make sure the modpacks category is always selected if it exists
	$: if (
		modpackCategoryExists &&
		selectedCategories &&
		!selectedCategories.some((category) => category?.slug === 'modpacks')
	) {
		selectedCategories = [
			$categories.find((category) => category.slug === 'modpacks')!,
			...selectedCategories
		];
	}

	$: includedFileCount = countIncludedFiles(includeFiles);

	function countIncludedFiles(includeFiles?: Map<string, boolean>) {
		if (!includeFiles) return 0;

		let count = 0;
		for (let enabled of includeFiles.values()) {
			if (enabled) count++;
		}
		return count;
	}

	async function refresh() {
		loading = 'Loading...';

		let args = await invokeCommand<ModpackArgs>('get_pack_args');

		name = args.name;
		author = args.author;
		nsfw = args.nsfw;
		description = args.description;
		selectedCategories = args.categories.map(
			(selected) => $categories.find((category) => category.slug === selected)!
		);
		changelog = args.changelog;
		readme = args.readme;
		versionNumber = args.versionNumber;
		iconPath = args.iconPath;
		websiteUrl = args.websiteUrl;
		includeDisabled = args.includeDisabled;
		includeFiles = new Map(Object.entries(args.includeFileMap));

		loading = null;
	}

	async function browseIcon() {
		let path = await open({
			defaultPath: iconPath.length > 0 ? iconPath : undefined,
			title: 'Select modpack icon',
			filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif'] }]
		});

		if (path === null) return;
		iconPath = path;
		saveArgs();
	}

	async function generateChangelog(all: boolean) {
		changelog = await invokeCommand('generate_changelog', { args: args(), all });
		saveArgs();
	}

	async function exportToFile() {
		let dir = await open({
			title: 'Choose folder to save modpack',
			defaultPath: `${name}.zip`,
			directory: true
		});

		if (!dir) return;

		loading = 'Exporting modpack to file...';
		try {
			await invokeCommand('export_pack', { args: args(), dir });
		} finally {
			loading = null;
		}
	}

	async function uploadToThunderstore() {
		let hasToken = await invokeCommand('has_thunderstore_token');

		if (!hasToken) {
			$apiKeyPopupOpen = true;

			await new Promise<void>((resolve) => {
				const interval = setInterval(() => {
					if (!$apiKeyPopupOpen) {
						clearInterval(interval);
						resolve();
					}
				}, 100);

				return () => clearInterval(interval);
			});

			hasToken = await invokeCommand('has_thunderstore_token');

			if (!hasToken) return;
		}

		loading = 'Uploading modpack to Thunderstore...';
		try {
			await invokeCommand('upload_pack', { args: args() });
			donePopupOpen = true;
		} finally {
			loading = null;
		}
	}

	function saveArgs() {
		// wait a tick to ensure the variables are updated
		setTimeout(() => {
			invokeCommand('set_pack_args', { args: args() });
		});
	}

	function args(): ModpackArgs {
		return {
			name,
			description,
			author,
			nsfw,
			readme,
			changelog,
			versionNumber,
			iconPath,
			websiteUrl,
			includeDisabled,
			includeFileMap: includeFiles,
			categories: selectedCategories.map(({ slug }) => slug)
		};
	}
</script>

<div class="relative mx-auto flex w-full max-w-4xl flex-col gap-1.5 overflow-y-auto px-6 py-4">
	{#if loading}
		<div
			class="text-primary-200 fixed inset-0 flex items-center justify-center bg-black/40 text-lg"
			transition:fade={{ duration: 50 }}
		>
			<Icon icon="mdi:loading" class="mr-4 animate-spin" />
			{loading}
		</div>
	{/if}

	<FormField
		label="Name"
		description="The name of the modpack, as shown on Thunderstore. Make sure this stays consistent between updates. Cannot contain spaces or hyphens."
		required={true}
	>
		<InputField
			on:change={saveArgs}
			bind:value={name}
			placeholder="Enter name..."
			required={true}
			pattern="^[a-zA-Z0-9_]+$"
			class="w-full"
		/>
	</FormField>

	<FormField
		label="Author"
		description="The name of the Thunderstore team connected to your API token."
		required={true}
	>
		<InputField
			on:change={saveArgs}
			bind:value={author}
			placeholder="Enter author..."
			required={true}
			class="w-full"
		/>
	</FormField>

	<FormField label="Description" description="A short description of the modpack." required={true}>
		<InputField
			on:change={saveArgs}
			bind:value={description}
			placeholder="Enter description..."
			required={true}
			maxlength={250}
			class="w-full"
		/>
	</FormField>

	<FormField
		label="Categories"
		description="The categories that the modpack belongs to. 'Modpacks' is always included."
	>
		<Dropdown
			avoidCollisions={false}
			items={$categories}
			bind:selected={selectedCategories}
			onSelectedChange={saveArgs}
			multiple={true}
			getLabel={(category) => category.name}
		>
			<Select.Trigger
				let:open
				slot="trigger"
				class="bg-primary-900 hover:border-primary-500 flex w-full items-center overflow-hidden rounded-lg border border-transparent py-1 pr-3 pl-1"
			>
				{#if selectedCategories.length === 0}
					<span class="text-primary-400 truncate pl-2">Select categories...</span>
				{:else}
					<div class="flex flex-wrap gap-1">
						{#each selectedCategories as category}
							<div class="bg-primary-800 text-primary-200 rounded-md py-1 pr-1 pl-3 text-sm">
								<span class="truncate overflow-hidden">{category.name}</span>

								<Button.Root
									class="hover:bg-primary-700 ml-1 rounded-md px-1.5"
									on:click={(evt) => {
										evt.stopPropagation();
										selectedCategories = selectedCategories.filter((c) => c !== category);
									}}
								>
									x
								</Button.Root>
							</div>
						{/each}
					</div>
				{/if}
				<Icon
					class="text-primary-400 ml-auto shrink-0 origin-center transform text-xl transition-all
                duration-100 ease-out {open ? 'rotate-180' : 'rotate-0'}"
					icon="mdi:chevron-down"
				/>
			</Select.Trigger>
		</Dropdown>
	</FormField>

	<FormField
		label="Version"
		description="The version number of the modpack, in the format of X.Y.Z.
			           You cannot publish with the same version number twice."
		required={true}
	>
		<InputField
			on:change={saveArgs}
			bind:value={versionNumber}
			placeholder="Enter version number..."
			required={true}
			pattern="^\d+\.\d+\.\d+$"
			class="w-full"
		/>
	</FormField>

	<FormField label="Website" description="The URL of a website of your choosing. Optional.">
		<InputField
			on:change={saveArgs}
			bind:value={websiteUrl}
			placeholder="Enter website URL..."
			pattern={URL_PATTERN}
			class="w-full"
		/>
	</FormField>

	<FormField
		label="Icon"
		description="Path to the icon of the modpack. This is automatically resized to 256x256 pixels, so
                 it's recommended to be a square image to avoid stretching or squishing."
		required={true}
	>
		<PathField icon="mdi:file-image" on:click={browseIcon} value={iconPath} />
	</FormField>

	<FormField
		label="Readme"
		description="A longer description of the modpack, which supports markdown formatting 
                 (similarly to Discord messages)."
		required={true}
	>
		<ResizableInputField
			on:change={saveArgs}
			bind:value={readme}
			placeholder="Enter readme..."
			mono={true}
		/>

		<details class="mt-1">
			<summary class="text-primary-300 cursor-pointer text-sm">Preview</summary>
			<Markdown class="mt-1 px-4" source={readme} />
			<div class="bg-primary-500 mt-4 h-[2px]" />
		</details>
	</FormField>

	<FormField
		label="Changelog"
		description="A list of changes in the modpack, also supports markdown formatting. Leave empty to omit."
	>
		<ResizableInputField
			on:change={saveArgs}
			bind:value={changelog}
			placeholder="Enter changelog..."
			mono={true}
		/>

		<BigButton color="primary" on:click={() => generateChangelog(false)}
			>Generate for {versionNumber}</BigButton
		>
		<BigButton color="primary" on:click={() => generateChangelog(true)}>Generate all</BigButton>

		<details class="mt-1">
			<summary class="text-primary-300 cursor-pointer text-sm">Preview</summary>
			<Markdown class="mt-1 px-4" source={changelog} />
			<div class="bg-primary-500 mt-4 h-[2px]" />
		</details>
	</FormField>

	<FormField
		label="Include files ({includedFileCount}/{includeFiles?.size})"
		description="Choose which config files to include in the modpack."
	>
		<details>
			{#if includeFiles}
				<summary class="text-primary-300 cursor-pointer text-sm">Show list</summary>
				<Checklist
					class="mt-1"
					title="Include all"
					items={Array.from(includeFiles.keys()).sort()}
					getLabel={(item) => item}
					get={(item) => includeFiles.get(item) ?? false}
					set={(item, _, value) => {
						includeFiles.set(item, value);
						includeFiles = includeFiles;
					}}
				/>
			{/if}
		</details>
	</FormField>

	<div class="text-primary-200 mt-1 flex items-center text-lg font-medium">
		<span class="max-w-96 grow">Contains NSFW content</span>

		<Checkbox onValueChanged={saveArgs} bind:value={nsfw} />
	</div>

	<div class="text-primary-200 flex items-center text-lg font-medium">
		<span class="max-w-96 grow">Include disabled mods</span>

		<Checkbox onValueChanged={saveArgs} bind:value={includeDisabled} />
	</div>

	<div class="mt-3 flex justify-end gap-2">
		<BigButton color="primary" on:click={exportToFile}>Export to file</BigButton>
		<BigButton color="accent" on:click={uploadToThunderstore}>Publish on Thunderstore</BigButton>
	</div>
</div>

<ApiKeyPopup />

<Popup bind:open={donePopupOpen} title="Modpack upload complete">
	<Dialog.Description class="text-primary-300">
		{name}
		{versionNumber} has successfully been published on Thunderstore!
		<Link href="https://thunderstore.io/c/{$activeGame?.slug}/p/{author}/{name}"
			>Click here to view its page on the website</Link
		>.
	</Dialog.Description>

	<div class="text-primary-400 mt-2 text-sm">
		The changes may take up to an hour to appear in Gale and other mod managers.
		<br />
		To publish a new update, increment the version number and publish the modpack again.
	</div>
</Popup>
