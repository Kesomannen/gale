<script lang="ts">
	import InputField from '$lib/components/ui/InputField.svelte';
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import ResizableInputField from '$lib/components/ui/ResizableInputField.svelte';
	import TabsMenu from '$lib/components/ui/TabsMenu.svelte';
	import { setConfigEntry } from '$lib/config';
	import { getListSeparator, type ListSeparator } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Tabs } from 'bits-ui';
	import { config } from '$lib/state/misc.svelte';

	let mode: 'text' | 'list' = $state('text');
	let newElement = $state('');
	let separator: ListSeparator = $state({ type: 'default', char: ',' });

	async function updateListContent() {
		content = items.join(separator.char);
		await submitValue();
	}

	async function submitValue() {
		if (!config.expandedEntry) return;

		await setConfigEntry(config.expandedEntry, {
			type: 'string',
			content
		});

		config.expandedEntry.entry.value.content = content;
	}

	function reset() {
		if (!config.expandedEntry) return;

		mode = 'text';
		newElement = '';

		content = config.expandedEntry.entry.value.content as string;
		separator = getListSeparator(config.expandedEntry.entry);
	}

	let open = $derived(config.expandedEntry !== null);

	let content = $derived((config.expandedEntry?.entry.value.content as string) ?? '');
	let items = $derived(content.split(separator.char));

	$effect(() => {
		if (open) reset();
	});
</script>

<Dialog
	large
	title="Edit {config.expandedEntry?.entry.name}"
	onclose={() => (config.expandedEntry = null)}
	{open}
>
	{#if config.expandedEntry && config.expandedEntry.entry.value.type === 'string'}
		<TabsMenu
			bind:value={mode}
			options={[
				{
					label: 'Edit as text',
					value: 'text'
				},
				{
					label: 'Edit as list',
					value: 'list'
				}
			]}
		>
			<Tabs.Content value="text" class="pt-1">
				<ResizableInputField
					value={content}
					onchange={(evt) => {
						content = evt.currentTarget.value;
						submitValue();
					}}
				/>
			</Tabs.Content>

			<Tabs.Content value="list" class="pt-1">
				<div class="text-primary-300 flex flex-col gap-1">
					{#each items as element, i}
						<div class="flex gap-1">
							<button
								class="text-primary-400 hover:bg-primary-700 hover:text-primary-300 rounded-lg p-1.5 text-xl"
								onclick={() => {
									items.splice(i, 1);
									updateListContent();
								}}
							>
								<Icon icon="mdi:remove" />
							</button>
							<InputField
								class="grow"
								value={element}
								onchange={(value) => {
									items[i] = value;
									updateListContent();
								}}
							/>
						</div>
					{/each}

					<InputField
						class="mt-1 w-full pr-9"
						placeholder="Enter new value..."
						bind:value={newElement}
						onchange={() => {
							if (newElement.length === 0) return;

							items.push(newElement);
							newElement = '';
							updateListContent();
						}}
					/>
				</div>
			</Tabs.Content>
		</TabsMenu>
	{/if}
</Dialog>
