import { zodResolver } from "@hookform/resolvers/zod"
import { useEffect, useState } from "react"
import { useForm } from "react-hook-form"
import { z } from "zod"
import { Button, buttonVariants } from "@/components/ui/button"
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form"
import { Input } from "@/components/ui/input"
import { Spinner } from "@/components/ui/spinner"
import { Clock } from "@/lib/formProperties"
import { cn } from "@/lib/utils"
import { ClockSchema } from "@/lib/api"
import { Pencil } from "lucide-react"
import { editUserClock } from "@/lib/api/clocks"

const editClockSchema = z.object({
	name: Clock.NAME.optional(),
})

export type EditClockProps = {
	clock: ClockSchema,
	onClockEditStart?: (changes: z.infer<typeof editClockSchema>) => void;
	onClockEdit?: (clock: ClockSchema | null) => void;
}

export default function EditClock(props: EditClockProps) {
	const [open, setOpen] = useState(false)
	const [editing, setEditing] = useState(false)

	const form = useForm<z.infer<typeof editClockSchema>>({
		resolver: zodResolver(editClockSchema),
		defaultValues: {
			name: props.clock.name
		}
	})

	useEffect(() => {
		form.formState.isSubmitSuccessful && form.reset()
	}, [form.formState.isSubmitSuccessful, form.reset])

	const submit = async (values: z.infer<typeof editClockSchema>) => {
		props?.onClockEditStart?.(values)
		setEditing(true)

		const clock = await editUserClock({
			userPoolId: props.clock.identity_pool_user_id,
			clockUuid: props.clock.uuid,
			edits: {
				name: values?.name
			}
		})

		props?.onClockEdit?.(clock)
		setOpen(false)
		setEditing(false)
		form.reset()
	}

	return (
		<Dialog open={open} onOpenChange={setOpen}>
			<DialogTrigger className={cn(buttonVariants({ variant: 'outline' }), 'self-center w-full mx-auto text-slate-950 gap-0')}>
				Edit "<span className="truncate">{props.clock.name}</span>" <Pencil strokeWidth={2} size={32} className="bg-lime-500 text-slate-100 rounded-full box-content ml-1 p-1" />
			</DialogTrigger>
			<DialogContent>
				<DialogHeader>
					<DialogTitle className="text-zinc-950">Edit Clock</DialogTitle>
					<DialogDescription>
						Enter optional fields below
					</DialogDescription>
				</DialogHeader>

				<Form {...form}>
					<form onSubmit={form.handleSubmit(submit)} className="space-y-8 text-zinc-950">
						<FormField
							disabled={editing}
							control={form.control}
							name="name"
							render={({ field }) => (
								<FormItem>
									<FormLabel>Name</FormLabel>
									<FormControl>
										<Input placeholder="e.g. My Fantastic Clock" {...field} />
									</FormControl>
									<FormDescription>
										A name to keep track of this clock
									</FormDescription>
									<FormMessage />
								</FormItem>
							)}
						/>

						<div className="flex items-center gap-4">
							<Button disabled={editing} type="submit">Submit</Button>
							{editing && <Spinner />}
						</div>
					</form>
				</Form>
			</DialogContent>
		</Dialog>
	)
}