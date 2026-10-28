import { Card } from "@heroui/react";

import { useT } from "../i18n/react";

type Props = {
  title: string;
  description: string;
};

export function PlaceholderScreen({ title, description }: Props) {
  const t = useT();
  return (
    <Card className="max-w-xl">
      <Card.Header>
        <Card.Title>{title}</Card.Title>
        <Card.Description>{t("app.historyNotReady")}</Card.Description>
      </Card.Header>
      <Card.Content>
        <p className="text-muted">{description}</p>
      </Card.Content>
    </Card>
  );
}
