import React from 'react'
import { SyntheticEvent, useState, FC } from 'react'
import styled from 'styled-components'
import { useForm, useMounted } from '@biotic-ui/std'
import { Input, Label, Section } from '@biotic-ui/input'
import { Button } from 'component/button'
import { useCreateProject, PartialProject } from 'data/project'
import { RequestError, ErrorCode } from 'data/http'

type Form = {
	name: string;
}

let DefaultForm = {
	name: ''
}

type Props = {
	onSuccess: (p: PartialProject) => void;
}

let CreateProject: FC<Props> = ({ onSuccess }) => {
	let isMounted = useMounted()
	let [error, setError] = useState<RequestError | null>(null)
	let [loading, setLoading] = useState<boolean>(false)

	let [form, setForm, set] = useForm<Form>(DefaultForm)

	let createProject = useCreateProject()

	async function handleSubmit(e: SyntheticEvent) {
		e.preventDefault()
		setLoading(true)

		try {
			let project = await createProject(form.name)
			if (isMounted) {
				setLoading(false)
				set(DefaultForm)
				onSuccess(project)
			}
		} catch (err) {
			if (isMounted) {
				setLoading(false)
				setError(err)
			}
		}
	}

	return (
		<Wrapper>
			<h1>Create New Project</h1>
			<form onSubmit={handleSubmit}>
				<Section>
					<Label>Project Name:</Label>
					<Input
						name='name'
						onChange={setForm}
						value={form.name}
						required
					/>
				</Section>
				<Section>
					<Button loading={loading}>
						Create Project
					</Button>
				</Section>

				{ error &&
					<Section>
						<p>{getErrorMessage(error)}</p>
					</Section>
				}
			</form>
		</Wrapper>
	)
}

export default CreateProject

let Wrapper = styled.div`
	padding: var(--baseline-3);
	padding-left: var(--baseline-4);
`

function getErrorMessage(error: RequestError): string {
	switch (error.code) {
		case ErrorCode.ProjectNameExists:
			return 'Project name already exists'

		default:
			return 'Something went wrong'
	}
}