<script lang="ts">
	import { Dialog } from 'bits-ui';
	import { fade, scale } from 'svelte/transition';
	import { quartIn, quartOut } from 'svelte/easing';
	import Icon from '@iconify/svelte';

	export let title: string | undefined = undefined;
	export let open: boolean;
	export let canClose: boolean = true;
	export let onClose: () => void = () => {};
</script>

<Dialog.Root
	bind:open
	closeOnEscape={canClose}
	onOutsideClick={(evt) => {
		if (!canClose) evt.preventDefault();
	}}
	onOpenChange={state => {
		if (!state) onClose();
	}}
>
	<Dialog.Portal>
		<Dialog.Overlay
			class="fixed inset-0 z-0 bg-black/60"
			transition={fade}
			transitionConfig={{ duration: 150 }}
		/>
		<Dialog.Content
			class="fixed inset-0 flex items-center justify-center pointer-events-none"
			inTransition={scale}
			inTransitionConfig={{ duration: 200, easing: quartOut, start: 0.8 }}
			outTransition={scale}
			outTransitionConfig={{ duration: 100, easing: quartIn, start: 0.8 }}
		>
			<div class="z-50 bg-gray-800 rounded-xl p-6 shadow-xl border border-gray-600 overflow-y-auto overflow-x-hidden max-h-[90%] pointer-events-auto
								 min-w-[45rem] w-fit max-w-[85%] relative">
				{#if title}
					<Dialog.Title class="w-full text-slate-100 font-bold text-2xl">{title}</Dialog.Title>
				{/if}

				<slot />

				<Dialog.Close class="absolute top-6 right-4 p-0.5 rounded-md text-slate-400 hover:text-slate-300 hover:bg-slate-700 text-3xl">
					<Icon icon="mdi:close" />
				</Dialog.Close>
			</div>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
