// API Response Types
export interface TradingPair {
  base: string;
  quote: string;
  volume_24h?: number;
}

export interface OrderbookEntry {
  price: string;
  amount: string;
}

export interface Orderbook {
  bids: OrderbookEntry[];
  asks: OrderbookEntry[];
}

export interface QuoteRequest {
  from_asset: string;
  to_asset: string;
  amount: string;
}

export interface QuoteResponse {
  from_asset: string;
  to_asset: string;
  input_amount: string;
  output_amount: string;
  price: string;
  route?: string[];
}
