<script lang="ts">
	import InputField from '$lib/components/ui/InputField.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import PathField from '$lib/components/ui/PathField.svelte';
	import Markdown from '$lib/components/ui/Markdown.svelte';
	import Link from '$lib/components/ui/Link.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import ApiKeyPopup, { apiKeyPopupOpen } from '$lib/components/prefs/ApiKeyPopup.svelte';

	import * as api from '$lib/api';
	import type { ModpackArgs } from '$lib/types';
	import { activeProfile, activeGame, categories } from '$lib/stores.svelte';
	import { open } from '@tauri-apps/plugin-dialog';

	import { Dialog } from 'bits-ui';
	import Popup from '$lib/components/ui/Popup.svelte';
	import Checklist from '$lib/components/ui/Checklist.svelte';
	import ResizableInputField from '$lib/components/ui/ResizableInputField.svelte';
	import { toHeaderCase } from 'js-convert-case';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import { SvelteMap } from 'svelte/reactivity';

	const URL_PATTERN =
		'[Hh][Tt][Tt][Pp][Ss]?://(?:(?:[a-zA-Z\u00a1-\uffff0-9]+-?)*[a-zA-Z\u00a1-\uffff0-9]+)(?:.(?:[a-zA-Z\u00a1-\uffff0-9]+-?)*[a-zA-Z\u00a1-\uffff0-9]+)*(?:.(?:[a-zA-Z\u00a1-\uffff]{2,}))(?::d{2,5})?(?:/[^s]*)?';

	let name: string = $state('');
	let author: string = $state('');
	let selectedCategories: string[] = $state([]);
	let nsfw: boolean = $state(false);
	let description: string = $state('');
	let readme: string = $state('');
	let changelog: string = $state('');
	let versionNumber: string = $state('');
	let iconPath: string = $state('');
	let websiteUrl: string = $state('');
	let includeDisabled: boolean = $state(false);
	let includeFiles = $state(new SvelteMap<string, boolean>());

	let donePopupOpen = $state(false);
	let loading: string | null = $state(null);

	let includedFileCount = $state(0);

	function countIncludedFiles(includeFiles?: SvelteMap<string, boolean>) {
		if (!includeFiles) return 0;

		let count = 0;
		for (let enabled of includeFiles.values()) {
			if (enabled) count++;
		}
		return count;
	}

	async function refresh() {
		loading = 'Loading...';

		let args = await api.profile.export.getPackArgs();

		name = args.name;
		author = args.author;
		nsfw = args.nsfw;
		description = args.description;
		selectedCategories = args.categories;
		changelog = args.changelog;
		readme = args.readme;
		versionNumber = args.versionNumber;
		iconPath = args.iconPath;
		websiteUrl = args.websiteUrl;
		includeDisabled = args.includeDisabled;
		includeFiles = new SvelteMap(Object.entries(args.includeFileMap));

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
		changelog = await api.profile.export.generateChangelog(args(), all);
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
			await api.profile.export.exportPack(dir, args());
		} finally {
			loading = null;
		}
	}

	async function uploadToThunderstore() {
		let hasToken = await api.thunderstore.hasToken();

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

			hasToken = await api.thunderstore.hasToken();

			if (!hasToken) return;
		}

		loading = 'Uploading modpack to Thunderstore...';
		try {
			await api.profile.export.uploadPack(args());
			donePopupOpen = true;
		} finally {
			loading = null;
		}
	}

	function saveArgs() {
		// wait a tick to ensure the variables are updated
		setTimeout(() => {
			api.profile.export.setPackArgs(args());
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
			categories: selectedCategories
		};
	}

	$effect(() => {
		$activeProfile;
		refresh();
	});

	// some communities don't have a specific modpack category
	let modpackCategoryExists = $derived(
		$categories.some((category) => category.slug === 'modpacks')
	);

	// make sure the modpacks category is always selected if it exists
	$effect(() => {
		if (
			modpackCategoryExists &&
			selectedCategories &&
			!selectedCategories.some((category) => category === 'modpacks')
		) {
			selectedCategories = ['modpacks', ...selectedCategories];
		}
	});

	$effect(() => {
		includedFileCount = countIncludedFiles(includeFiles);
	});
</script>

<div class="relative mx-auto flex w-full max-w-4xl flex-col gap-1.5 overflow-y-auto px-6 py-4">
	{#if loading}
		<div class="text-primary-200 absolute inset-0 flex items-center justify-center gap-2 text-lg">
			<Spinner />
			{loading}
		</div>
	{:else}
		<FormField
			label="Name"
			description="The name of the modpack, as shown on Thunderstore. Make sure this stays consistent between updates. Cannot contain spaces or hyphens."
			required={true}
		>
			<InputField
				onchange={saveArgs}
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
			required
		>
			<InputField
				onchange={saveArgs}
				bind:value={author}
				placeholder="Enter author..."
				class="w-full"
				required
			/>
		</FormField>

		<FormField label="Description" description="A short description of the modpack." required>
			<InputField
				onchange={saveArgs}
				bind:value={description}
				placeholder="Enter description..."
				maxlength={250}
				class="w-full"
				required
			/>
		</FormField>

		<FormField
			label="Categories"
			description="The categories that the modpack belongs to. 'Modpacks' is always included."
		>
			<Select
				items={$categories.map((category) => ({
					label: category.name,
					value: category.slug
				}))}
				bind:value={selectedCategories}
				onValueChange={saveArgs}
				type="multiple"
				triggerClass="w-full"
			>
				{#snippet label()}
					{#if selectedCategories.length === 0}
						<span class="text-primary-400 truncate pl-2">Select categories...</span>
					{:else}
						<div class="flex flex-wrap gap-1">
							{#each selectedCategories as category}
								<div class="bg-primary-800 text-primary-200 rounded-md py-1 pr-1 pl-3 text-sm">
									<span class="truncate overflow-hidden">{toHeaderCase(category)}</span>

									<button
										class="hover:bg-primary-700 ml-1 rounded-md px-1.5"
										onclick={(evt) => {
											evt.stopPropagation();
											selectedCategories = selectedCategories.filter((cat) => cat !== category);
										}}
									>
										x
									</button>
								</div>
							{/each}
						</div>
					{/if}
				{/snippet}
			</Select>
		</FormField>

		<FormField
			label="Version"
			description="The version number of the modpack, in the format of X.Y.Z.
			           You cannot publish with the same version number twice."
			required
		>
			<InputField
				onchange={saveArgs}
				bind:value={versionNumber}
				placeholder="Enter version number..."
				required={true}
				pattern="^\d+\.\d+\.\d+$"
				class="w-full"
			/>
		</FormField>

		<FormField label="Website" description="The URL of a website of your choosing. Optional.">
			<InputField
				onchange={saveArgs}
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
			required
		>
			<PathField icon="mdi:file-image" onclick={browseIcon} value={iconPath} />
		</FormField>

		<FormField
			label="Readme"
			description="A longer description of the modpack, which supports markdown formatting (similarly to Discord messages)."
			required
		>
			<ResizableInputField
				onchange={saveArgs}
				bind:value={readme}
				placeholder="Enter readme..."
				mono={true}
			/>

			<details class="mt-1">
				<summary class="text-primary-300 cursor-pointer text-sm">Preview</summary>
				<Markdown class="mt-1 px-4" source={readme} />
				<div class="bg-primary-500 mt-4 h-[2px]"></div>
			</details>
		</FormField>

		<FormField
			label="Changelog"
			description="A list of changes in the modpack, also supports markdown formatting. Leave empty to omit."
		>
			<ResizableInputField
				onchange={saveArgs}
				bind:value={changelog}
				placeholder="Enter changelog..."
				mono={true}
			/>

			<Button color="primary" onclick={() => generateChangelog(false)}
				>Generate for {versionNumber}</Button
			>
			<Button color="primary" onclick={() => generateChangelog(true)}>Generate all</Button>

			<details class="mt-1">
				<summary class="text-primary-300 cursor-pointer text-sm">Preview</summary>
				<Markdown class="mt-1 px-4" source={changelog} />
				<div class="bg-primary-500 mt-4 h-[2px]"></div>
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

			<Checkbox onCheckedChange={saveArgs} bind:checked={nsfw} />
		</div>

		<div class="text-primary-200 flex items-center text-lg font-medium">
			<span class="max-w-96 grow">Include disabled mods</span>

			<Checkbox onCheckedChange={saveArgs} bind:checked={includeDisabled} />
		</div>

		<div class="mt-3 flex justify-end gap-2">
			<Button color="primary" onclick={exportToFile}>Export to file</Button>
			<Button color="accent" onclick={uploadToThunderstore}>Publish on Thunderstore</Button>
		</div>
	{/if}
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
