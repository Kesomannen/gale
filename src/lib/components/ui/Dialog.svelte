<script lang="ts">
	import { Dialog } from 'bits-ui';
	import { fade } from 'svelte/transition';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';

	import type { Snippet } from 'svelte';

	type Props = {
		open: boolean;
		title?: string | null;
		confirmClose?: { message: string } | null;
		canClose?: boolean;
		large?: boolean;
		noscroll?: boolean;
		onclose?: () => void;
		children?: Snippet;
	};

	let {
		open = $bindable(),
		title = null,
		confirmClose = null,
		canClose = true,
		large = false,
		noscroll = false,
		onclose,
		children
	}: Props = $props();

	async function close(evt: UIEvent) {
		if (!canClose) {
			evt.preventDefault();
			return;
		}

		if (confirmClose) {
			evt.preventDefault();
			let result = await confirm(confirmClose.message);
			if (!result) return;
		}

		open = false;
		onclose?.();
	}
</script>

<Dialog.Root
	bind:open
	onOpenChange={(open) => {
		if (!open) onclose?.();
	}}
>
	<Dialog.Portal>
		<Dialog.Overlay forceMount class="pointer-events-none" data-tauri-drag-region={!canClose}>
			{#snippet child({ props: { style, ...props }, open })}
				{#if open}
					<div
						{...props}
						transition:fade={{ duration: 80 }}
						class="fixed inset-0 z-0 rounded-lg"
						style="background-color: rgba(0, 0, 0, calc(60%/(var(--bits-dialog-depth) + var(--bits-dialog-nested-count) + 1))); {style}"
					></div>
				{/if}
			{/snippet}
		</Dialog.Overlay>
		<Dialog.Content
			interactOutsideBehavior={canClose && confirmClose === null ? 'close' : 'ignore'}
			class="pointer-events-none"
		>
			{#if open}
				<div class="pointer-events-none fixed inset-0 flex items-center justify-center">
					<div
						class={[
							large ? 'max-w-240' : 'max-w-140',
							noscroll ? 'flex flex-col overflow-y-hidden' : 'overflow-y-auto',
							'border-primary-300 dark:border-primary-600 dark:bg-primary-800 pointer-events-auto relative z-30 max-h-[85%] w-[85%] overflow-x-hidden rounded-xl border bg-white p-6 shadow-xl'
						]}
					>
						{#if title}
							<Dialog.Title
								class="text-primary-900 w-full pr-10 text-2xl font-bold wrap-break-word dark:text-white"
								>{title}</Dialog.Title
							>
						{/if}

						{@render children?.()}

						{#if canClose}
							<button
								class="text-primary-500 hover:text-primary-700 dark:text-primary-400 dark:hover:bg-primary-700 dark:hover:text-primary-300 hover:bg-primary-200 absolute top-5 right-5 rounded-md p-0.5 text-3xl"
								onclick={close}
							>
								<Icon icon="mdi:close" />
							</button>
						{/if}
					</div>
				</div>
			{/if}
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
