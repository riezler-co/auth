import React from 'react'

import { createContext, useContext, FC, useMemo, useEffect } from 'react'
import Axios, { AxiosInstance, AxiosError, AxiosResponse } from 'axios'
import { AuthClient } from '@riezler/auth-sdk'

export let CancelToken = Axios.CancelToken

let HttpCtx = createContext<AxiosInstance>(Axios)

type Props = {
	project?: string | null;
	auth: AuthClient | null;
}

export let Http: FC<Props> = ({ auth, project, children }) => {
	let httpClient = useMemo(() => {

		if (!auth || !project) {
			return Axios
		}

		let instance = Axios.create({
			baseURL: '/',
			headers: {
				'Bento-Project': project
			},
		})

		instance.interceptors.request.use(async function (config) {
			try {
				let token = await auth
					.getToken()

				return {
					...config,
					headers: {
						...config.headers,
						'Authorization': `Bearer ${token}`,
					}
				};
			} catch (err) {
				throw new Axios.Cancel(err.message)
			}
		})

		return instance
	}, [project, auth])

	return (
		<HttpCtx.Provider value={httpClient}>
			{ children }
		</HttpCtx.Provider>
	)
}

export function useHttp() {
	return useContext(HttpCtx)
}
