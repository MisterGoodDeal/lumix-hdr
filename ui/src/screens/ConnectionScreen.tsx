import { Alert, Button, Card, Chip } from "@heroui/react";

import { ErrorDetails } from "../components/ErrorDetails";
import {
  connectionHelpText,
  connectionStatusLabel,
  osLabel,
} from "../lib/connection-status";
import { useAppStore } from "../store/app-store";

export function ConnectionScreen() {
  const status = useAppStore((state) => state.status);
  const error = useAppStore((state) => state.error);
  const demoConnected = useAppStore((state) => state.demoConnected);
  const connectDemo = useAppStore((state) => state.connectDemo);

  if (!status) {
    return (
      <Card className="max-w-xl">
        <Card.Header>
          <Card.Title>Connexion</Card.Title>
          <Card.Description>Chargement de l'état de l'application…</Card.Description>
        </Card.Header>
        <Card.Content>
          <ErrorDetails error={error} />
        </Card.Content>
      </Card>
    );
  }

  const label = connectionStatusLabel(status, demoConnected);
  const help = connectionHelpText(status);
  const chipColor = status.demo_mode
    ? "accent"
    : status.camera_available
      ? "warning"
      : "danger";

  return (
    <div className="flex max-w-2xl flex-col gap-4">
      <Card>
        <Card.Header>
          <Card.Title>Connexion</Card.Title>
          <Card.Description>
            LumixHDR {status.version} · {osLabel(status.os)}
          </Card.Description>
        </Card.Header>
        <Card.Content className="flex flex-col gap-4">
          <div className="flex items-center gap-3">
            <Chip color={chipColor} variant="soft">
              <Chip.Label>{label}</Chip.Label>
            </Chip>
          </div>
          <p>{help}</p>
          {status.demo_mode ? (
            <Button
              variant="primary"
              isDisabled={demoConnected}
              onPress={connectDemo}
            >
              {demoConnected ? "Démo connectée" : "Connecter"}
            </Button>
          ) : (
            <Button variant="primary" isDisabled>
              Connecter
            </Button>
          )}
          <ErrorDetails error={error} />
        </Card.Content>
      </Card>
      {!status.camera_available && !status.demo_mode ? (
        <Alert status="warning">
          <Alert.Indicator />
          <Alert.Content>
            <Alert.Title>Windows requis pour le GH6</Alert.Title>
            <Alert.Description>
              Le SDK LUMIX n'est disponible que sous Windows 10/11 64 bits, en USB
              direct. Activez le mode démo dans Réglages pour tester l'interface
              sur cet ordinateur.
            </Alert.Description>
          </Alert.Content>
        </Alert>
      ) : null}
    </div>
  );
}
