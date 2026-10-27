import { Card } from "@heroui/react";

type Props = {
  title: string;
  description: string;
};

export function PlaceholderScreen({ title, description }: Props) {
  return (
    <Card className="max-w-xl">
      <Card.Header>
        <Card.Title>{title}</Card.Title>
        <Card.Description>Pas encore disponible</Card.Description>
      </Card.Header>
      <Card.Content>
        <p className="text-muted">{description}</p>
      </Card.Content>
    </Card>
  );
}
