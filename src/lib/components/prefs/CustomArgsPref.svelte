<script lang="ts">
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import Info from '$lib/components/ui/Info.svelte';
	import InputField from '$lib/components/ui/InputField.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '@iconify/svelte';
	import type { DragEventHandler } from 'svelte/elements';

	type Props = {
		value: string[];
		enabled: boolean;
		setValue: (value: string[]) => Promise<void>;
		setEnabled: (enabled: boolean) => Promise<void>;
	};

	let { value = $bindable(), enabled = $bindable(), setValue, setEnabled }: Props = $props();

	let newArg = $state('');

	let reorderPrevIndex: number | null = null;

	const ondragstart: DragEventHandler<HTMLDivElement> = (evt) => {
		const element = evt.currentTarget;
		reorderPrevIndex = parseInt(element.dataset.index!);
		evt.dataTransfer!.effectAllowed = 'move';
		evt.dataTransfer!.setData('text/html', element.outerHTML);
	};

	const ondragover: DragEventHandler<HTMLDivElement> = async (evt) => {
		evt.preventDefault();
		if (reorderPrevIndex === null) return;

		const element = evt.currentTarget;
		const newIndex = parseInt(element.dataset.index!);

		if (newIndex === reorderPrevIndex) {
			return;
		}

		const temp = value[reorderPrevIndex];
		value[reorderPrevIndex] = value[newIndex];
		value[newIndex] = temp;

		reorderPrevIndex = newIndex;
	};

	const ondragend: DragEventHandler<HTMLDivElement> = async () => {
		if (reorderPrevIndex !== null) {
			await setValue(value);
		}
		reorderPrevIndex = null;
	};
</script>

<div class="mt-1 flex items-center">
	<Label>{m.customArgsPref_title()}</Label>

	<Info>
		<p>
			{m.customArgsPref_content_1()}<b>{m.customArgsPref_content_2()}</b
			>{m.customArgsPref_content_3()}
		</p>

		<p>
			{m.customArgsPref_content_4()}
			<code>--foo value</code>
			{m.customArgsPref_content_5()}
			<code>--foo</code>
			{m.customArgsPref_content_6()}
			<code>value</code>
			{m.customArgsPref_content_7()}
		</p>
	</Info>

	<Checkbox
		checked={enabled}
		onCheckedChange={(checked) => {
			setEnabled(checked);
		}}
	/>
</div>

{#if enabled}
	<div class="text-primary-300 mt-1 flex flex-col gap-1 pl-[35%]" role="list">
		{#each value as argument, i}
			<div
				role="listitem"
				class="flex gap-1 rounded-lg border-2 border-transparent transition-colors"
				draggable={true}
				data-index={i}
				{ondragstart}
				{ondragover}
				{ondragend}
			>
				<button
					class="text-primary-400 hover:bg-primary-700 hover:text-primary-300 rounded-lg p-1.5 text-xl"
					onclick={() => {
						value = value.filter((_, index) => index !== i);
						setValue(value);
					}}
				>
					<Icon icon="mdi:remove" />
				</button>
				<InputField
					class="grow"
					value={argument}
					onchange={(newValue) => {
						value[i] = newValue;
						setValue([...value]);
					}}
				/>
				{#if value.length > 1}
					<button
						class="text-primary-400 hover:bg-primary-700 hover:text-primary-300 cursor-move rounded-lg p-1.5 text-xl"
					>
						<Icon icon="material-symbols:drag-indicator" />
					</button>
				{/if}
			</div>
		{/each}

		<InputField
			placeholder={m.customArgsPref_placeholder()}
			bind:value={newArg}
			onchange={() => {
				if (newArg.length === 0) return;

				value = [...value, newArg];
				newArg = '';
				setValue(value);
			}}
		/>
	</div>
{/if}
