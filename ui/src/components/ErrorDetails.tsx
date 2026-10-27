import { Accordion } from "@heroui/react";

import type { AppError } from "../types";

type Props = {
  error: AppError | null;
};

export function ErrorDetails({ error }: Props) {
  if (!error) {
    return null;
  }

  return (
    <div className="mt-4">
      <p className="text-danger">{error.message}</p>
      {error.detail ? (
        <Accordion className="mt-2">
          <Accordion.Item id="detail">
            <Accordion.Heading>
              <Accordion.Trigger>
                Détail technique
                <Accordion.Indicator />
              </Accordion.Trigger>
            </Accordion.Heading>
            <Accordion.Panel>
              <Accordion.Body>
                <pre className="whitespace-pre-wrap text-sm text-muted">
                  {error.detail}
                </pre>
              </Accordion.Body>
            </Accordion.Panel>
          </Accordion.Item>
        </Accordion>
      ) : null}
    </div>
  );
}
