import React, { useState } from "react";
import { invoke } from "@tauri-apps/api";
import { QrCode } from "lucide-react";
import { Button } from "./ui/button";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "./ui/dialog";
import QRCode from "react-qr-code";

interface ConnectWithQRProps {
  isOpen: boolean;
  onOpenChange: (isOpen: boolean) => void;
}

const ConnectWithQR: React.FC<ConnectWithQRProps> = ({ isOpen, onOpenChange }) => {
  const [isQrOpen, setIsQrOpen] = useState(false);
  const [qrData, setQrData] = useState<string | null>(null);

  React.useEffect(() => {
    if (isOpen) {
      fetchQRData();
    }
  }, [isOpen]);

  const fetchQRData = async () => {
    try {
      const config = await invoke<{ ip: string; port: number }>("get_server_config");
      setQrData(`${config.ip}:${config.port}`);
    } catch (error) {
      console.error("Failed to fetch QR data:", error);
    }
  };

  return (
    <div>
      {/* Button to open QR dialog */}
      <Button variant="outline" onClick={() => {
        setIsQrOpen(true)
        fetchQRData()}
        }>
        <QrCode className="h-4 w-4 mr-1" />
        Connect with QR
      </Button>

      {/* QR Code Dialog */}
      <Dialog open={isQrOpen} onOpenChange={onOpenChange}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Connect with QR</DialogTitle>
          </DialogHeader>
          <div className="flex flex-col justify-center items-center gap-8">
            {qrData ? (
              <>
                <div className="flex flex-col justify-center items-center gap-4">
                    <p className="text-sm text-gray-500">Scan with Button Beam mobile app</p>
                    <QRCode value={qrData} />
                    {/* Display IP and Port below the QR code */}
                    <p className="text-sm text-gray-500">Or enter manually: {' '}
                        <span className="bg-gray-100 font-mono p-1 px-2 text-gray-700">
                        {qrData}
                        </span>
                    </p>
                </div>
              </>
            ) : (
              <p>Loading QR Code...</p>
            )}
            <Button onClick={() => setIsQrOpen(false)}>Close</Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default ConnectWithQR;
