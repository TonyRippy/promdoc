// State tracking for the promdoc client UI.
// Copyright (C) 2023, Tony Rippy
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

import { PrometheusDriver } from 'prometheus-query';

interface ClientConfig {
  prometheus_urls: string[]
}

export class Client {
  private prom: PrometheusDriver[] = []

  constructor (config: ClientConfig) {
    config.prometheus_urls.forEach(url => {
      this.prom.push(new PrometheusDriver({ endpoint: url }))
    })
  }
}

export async function init(): Promise<Client> {
  const config = await fetch('/config')
    .then(res => res.json())
    .then(res => {
      return res as ClientConfig
    })
  return new Client(config)
}
