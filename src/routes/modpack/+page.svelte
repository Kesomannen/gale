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
	import { fade } from 'svelte/transition';
	import Icon from '@iconify/svelte';

	import { Button, Dialog, Select } from 'bits-ui';
	import Popup from '$lib/components/Popup.svelte';
	import Checklist from '$lib/components/Checklist.svelte';
	import ResizableInputField from '$lib/components/ResizableInputField.svelte';

	import { t, T } from '$i18n';

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

	// make sure 'Modpacks' category is always selected
	$: if (
		selectedCategories &&
		!selectedCategories.some((category) => category?.name === 'Modpacks')
	) {
		selectedCategories = [
			$categories.find((category) => category.name === 'Modpacks')!,
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
		loading = t('Loading');

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
		let response = await open({
			defaultPath: iconPath.length > 0 ? iconPath : undefined,
			title: t('Select modpack icon'),
			filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif'] }]
		});

		if (!response) return;
		iconPath = response.path;
		saveArgs();
	}

	async function generateChangelog(all: boolean) {
		changelog = await invokeCommand('generate_changelog', { args: args(), all });
		saveArgs();
	}

	async function exportToFile() {
		let dir = await open({
			title: t('Choose directory save modpack'),
			defaultPath: `${name}.zip`,
			directory: true
		});

		if (!dir) return;

		loading = t('Exporting modpack to file');
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

		loading = t('Uploading modpack to Thunderstore');
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

<div class="relative flex w-full flex-col gap-1.5 overflow-y-auto px-6 py-4">
	{#if loading}
		<div
			class="fixed inset-0 flex items-center justify-center bg-black/40 text-lg text-slate-200"
			transition:fade={{ duration: 50 }}
		>
			<Icon icon="mdi:loading" class="mr-4 animate-spin" />
			{loading}
		</div>
	{/if}

	<FormField
		label="{t("Name")}"
		description="{t("Modepack name description")}"
		required={true}
	>
		<InputField
			on:change={saveArgs}
			bind:value={name}
			placeholder="{t("Enter name")}"
			required={true}
			pattern="^[a-zA-Z0-9_]+$"
			class="w-full"
		/>
	</FormField>

	<FormField
		label="{t("Author")}"
		description="{t("Modepack author description")}"
		required={true}
	>
		<InputField
			on:change={saveArgs}
			bind:value={author}
			placeholder={t("Enter author")}
			required={true}
			class="w-full"
		/>
	</FormField>

	<FormField label="{t("Description")}" description="{t("Modepack description description")}" required={true}>
		<InputField
			on:change={saveArgs}
			bind:value={description}
			placeholder="{t("Enter description")}"
			required={true}
			maxlength={250}
			class="w-full"
		/>
	</FormField>

	<FormField
		label="{t("Categories")}"
		description="{t("Modepack categories description")}"
	>
		{#if selectedCategories}
			<Dropdown
				avoidCollisions={false}
				items={$categories}
				bind:selected={selectedCategories}
				onSelectedChange={saveArgs}
				multiple={true}
				getLabel={(category) => category?.name || "Unknown"}
			>
				<Select.Trigger
					let:open
					slot="trigger"
					class="flex w-full items-center overflow-hidden rounded-lg border border-gray-500 border-opacity-0 bg-gray-900 py-1 pl-1 pr-3 hover:border-opacity-100"
				>
					{#if selectedCategories.length === 0}
						<span class="truncate pl-2 text-slate-400">{t("Select categories")}</span>
					{:else}
						<div class="flex flex-wrap gap-1">
							{#each selectedCategories as category}
								<div class="rounded-md bg-gray-800 py-1 pl-3 pr-1 text-sm text-slate-200">
									<span class="overflow-hidden truncate">{category.name}</span>

									<Button.Root
										class="ml-1 rounded-md px-1.5 hover:bg-gray-700"
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
						class="ml-auto flex-shrink-0 origin-center transform text-xl text-slate-400 transition-all
                duration-100 ease-out {open ? 'rotate-180' : 'rotate-0'}"
						icon="mdi:chevron-down"
					/>
				</Select.Trigger>
			</Dropdown>
		{/if}
	</FormField>

	<FormField
		label="{t("Version")}"
		description="{t("Modepack version description")}"
		required={true}
	>
		<InputField
			on:change={saveArgs}
			bind:value={versionNumber}
			placeholder="{t("Enter version number")}"
			required={true}
			pattern="^\d+\.\d+\.\d+$"
			class="w-full"
		/>
	</FormField>

	<FormField label="{t("Website")}" description="{t("Modepack websiter description")}">
		<InputField
			on:change={saveArgs}
			bind:value={websiteUrl}
			placeholder="{t("Enter website URL")}"
			pattern={URL_PATTERN}
			class="w-full"
		/>
	</FormField>

	<FormField
		label="{t("Icon")}"
		description="{t("Modepack icon description")}"
		required={true}
	>
		<PathField icon="mdi:file-image" onClick={browseIcon} value={iconPath} />
	</FormField>

	<FormField
		label="{t("Readme")}"
		description="{t("Modepack readme description")}"
		required={true}
	>
		<ResizableInputField
			on:change={saveArgs}
			bind:value={readme}
			placeholder="Enter readme..."
			mono={true}
		/>

		<details class="mt-1">
			<summary class="cursor-pointer text-sm text-slate-300">Preview</summary>
			<Markdown class="mt-1 px-4" source={readme} />
			<div class="mt-4 h-[2px] bg-gray-500" />
		</details>
	</FormField>

	<FormField
		label="{t("Changelog")}"
		description="{t("Modepack changelog description")}"
	>
		<ResizableInputField
			on:change={saveArgs}
			bind:value={changelog}
			placeholder="Enter changelog..."
			mono={true}
		/>

		<BigButton color="gray" on:click={() => generateChangelog(false)}
			>{T("Generate for version number", {"versionNumber": versionNumber})}</BigButton
		>
		<BigButton color="gray" on:click={() => generateChangelog(true)}>{t("Generate all")}</BigButton>

		<details class="mt-1">
			<summary class="cursor-pointer text-sm text-slate-300">{t("Preview")}</summary>
			<Markdown class="mt-1 px-4" source={changelog} />
			<div class="mt-4 h-[2px] bg-gray-500" />
		</details>
	</FormField>

	<FormField
		label= "{t("Include files")} ({includedFileCount}/{includeFiles?.size})"
		description={t("Modepack include description")}
	>
		<details>
			{#if includeFiles}
				<summary class="cursor-pointer text-sm text-slate-300">{t("Show list")}</summary>
				<Checklist
					class="mt-1"
					title={t("Include all")}
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

	<div class="mt-1 flex items-center text-lg font-medium text-slate-200">
		<span class="max-w-96 flex-grow">{t("Contains NSFW content")}</span>

		<Checkbox onValueChanged={saveArgs} bind:value={nsfw} />
	</div>

	<div class="flex items-center text-lg font-medium text-slate-200">
		<span class="max-w-96 flex-grow">{t("Include disabled mods")}</span>

		<Checkbox onValueChanged={saveArgs} bind:value={includeDisabled} />
	</div>

	<div class="mt-3 flex justify-end gap-2">
		<BigButton color="gray" on:click={exportToFile}>{t("Export to file")}</BigButton>
		<BigButton color="green" on:click={uploadToThunderstore}>{t("Publish on Thunderstore")}</BigButton>
	</div>
</div>

<ApiKeyPopup />

<Popup bind:open={donePopupOpen} title="{t("Modpack upload complete")}">
	<Dialog.Description class="text-slate-300">
		{name}
		{versionNumber} has successfully been published on Thunderstore!
		{t("Modpack upload complete description 1")}
		<Link href="https://thunderstore.io/c/{$activeGame?.id}/p/{author}/{name}">
			{t("Modpack upload complete description 2")}
		</Link>
		{t("Modpack upload complete description 3")}
	</Dialog.Description>

	<div class="mt-2 text-sm text-slate-400">
		{t("Modpack upload complete description 4")}
		<br />
		{t("Modpack upload complete description 5")}
	</div>
</Popup>
