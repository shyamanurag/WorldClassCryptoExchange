

import React, { useState, useEffect } from 'react';
import { Card, CardContent, Typography, Box, Grid, Button, Chip, CircularProgress, Divider, Paper, Table, TableBody, TableCell, TableContainer, TableHead, TableRow } from '@mui/material';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { formatCurrency, formatPercentage } from '../../utils/formatters';
import { useAuth } from '../../context/AuthContext';
import { DeFiService } from '../../services/DeFiService';

// Types
interface Position {
  id: string;
  protocol_id: string;
  pool_id: string | null;
  assets_deposited: AssetAmount[];
  tokens_received: AssetAmount[];
  value_usd: number;
  apy: number;
  status: 'Active' | 'Closing' | 'Closed' | 'Failed';
  created_at: string;
  updated_at: string;
  closed_at: string | null;
}

interface AssetAmount {
  asset_id: string;
  amount: number;
}

interface Protocol {
  id: string;
  name: string;
  protocol_type: string;
  description: string;
  risk_level: 'Low' | 'Medium' | 'High' | 'VeryHigh';
  tvl: number;
  apy_range: [number, number];
  is_active: boolean;
}

interface Pool {
  id: string;
  protocol_id: string;
  name: string;
  assets: string[];
  tvl: number;
  apy: number;
  is_active: boolean;
  risk_level: 'Low' | 'Medium' | 'High' | 'VeryHigh';
}

interface HealthStatus {
  position_id: string;
  health_factor: number;
  status: string;
}

// Main DeFi Portfolio Component
const DeFiPortfolio: React.FC = () => {
  const { user } = useAuth();
  const [positions, setPositions] = useState<Position[]>([]);
  const [protocols, setProtocols] = useState<Protocol[]>([]);
  const [portfolioValue, setPortfolioValue] = useState<number>(0);
  const [healthStatuses, setHealthStatuses] = useState<HealthStatus[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [historicalData, setHistoricalData] = useState<any[]>([]);

  useEffect(() => {
    const fetchData = async () => {
      if (!user) return;
      
      setLoading(true);
      try {
        // Fetch protocols
        const protocolsData = await DeFiService.listProtocols();
        setProtocols(protocolsData.protocols);
        
        // Fetch user positions
        const positionsData = await DeFiService.listPositions(user.id);
        setPositions(positionsData.positions);
        
        // Fetch portfolio value
        const valueData = await DeFiService.getPortfolioValue(user.id);
        setPortfolioValue(valueData.value);
        
        // Fetch health statuses
        const healthData = await DeFiService.checkPositionsHealth(user.id);
        setHealthStatuses(healthData.positions);
        
        // For demo purposes, generate historical data
        generateHistoricalData();
      } catch (error) {
        console.error("Error fetching DeFi portfolio data:", error);
      } finally {
        setLoading(false);
      }
    };
    
    fetchData();
  }, [user]);
  
  const generateHistoricalData = () => {
    const now = new Date();
    const data = [];
    
    // Generate 30 days of data
    for (let i = 30; i >= 0; i--) {
      const date = new Date(now);
      date.setDate(date.getDate() - i);
      
      // Simulate growing portfolio with some volatility
      const baseValue = 10000; // Starting value
      const growthFactor = 1 + (30 - i) * 0.01; // 1% growth per day
      const volatility = 0.05; // 5% max volatility
      const randomFactor = 1 + (Math.random() * 2 - 1) * volatility;
      
      data.push({
        date: date.toISOString().split('T')[0],
        value: baseValue * growthFactor * randomFactor,
      });
    }
    
    setHistoricalData(data);
  };
  
  const getProtocolById = (id: string): Protocol | undefined => {
    return protocols.find(p => p.id === id);
  };
  
  const getActivePositionsCount = (): number => {
    return positions.filter(p => p.status === 'Active').length;
  };
  
  const getTotalInvested = (): number => {
    return positions.reduce((total, position) => {
      if (position.status === 'Active') {
        return total + position.value_usd;
      }
      return total;
    }, 0);
  };
  
  const getAverageAPY = (): number => {
    const activePositions = positions.filter(p => p.status === 'Active');
    if (activePositions.length === 0) return 0;
    
    const totalAPY = activePositions.reduce((sum, position) => sum + position.apy, 0);
    return totalAPY / activePositions.length;
  };
  
  const handleHarvestRewards = async (positionId: string) => {
    if (!user) return;
    
    try {
      await DeFiService.harvestRewards(user.id, positionId);
      // Refresh positions after harvesting
      const positionsData = await DeFiService.listPositions(user.id);
      setPositions(positionsData.positions);
    } catch (error) {
      console.error("Error harvesting rewards:", error);
    }
  };
  
  const handleClosePosition = async (positionId: string) => {
    if (!user) return;
    
    try {
      await DeFiService.closePosition(user.id, positionId);
      // Refresh positions after closing
      const positionsData = await DeFiService.listPositions(user.id);
      setPositions(positionsData.positions);
      
      // Update portfolio value
      const valueData = await DeFiService.getPortfolioValue(user.id);
      setPortfolioValue(valueData.value);
    } catch (error) {
      console.error("Error closing position:", error);
    }
  };
  
  if (loading) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" height="50vh">
        <CircularProgress />
      </Box>
    );
  }
  
  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        DeFi Portfolio
      </Typography>
      
      {/* Portfolio Overview */}
      <Grid container spacing={3} mb={4}>
        <Grid item xs={12} md={6} lg={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom>
                Portfolio Value
              </Typography>
              <Typography variant="h4">
                {formatCurrency(portfolioValue)}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} md={6} lg={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom>
                Active Positions
              </Typography>
              <Typography variant="h4">
                {getActivePositionsCount()}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} md={6} lg={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom>
                Total Invested
              </Typography>
              <Typography variant="h4">
                {formatCurrency(getTotalInvested())}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} md={6} lg={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom>
                Average APY
              </Typography>
              <Typography variant="h4">
                {formatPercentage(getAverageAPY())}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
      </Grid>
      
      {/* Portfolio Chart */}
      <Card sx={{ mb: 4 }}>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Portfolio Performance
          </Typography>
          <Box height={300}>
            <ResponsiveContainer width="100%" height="100%">
              <LineChart
                data={historicalData}
                margin={{ top: 5, right: 30, left: 20, bottom: 5 }}
              >
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis 
                  dataKey="date" 
                  tickFormatter={(value) => {
                    const date = new Date(value);
                    return `${date.getMonth() + 1}/${date.getDate()}`;
                  }}
                />
                <YAxis 
                  tickFormatter={(value) => `$${value.toLocaleString()}`}
                />
                <Tooltip 
                  formatter={(value: any) => [`$${value.toLocaleString()}`, 'Value']}
                  labelFormatter={(label) => `Date: ${label}`}
                />
                <Legend />
                <Line 
                  type="monotone" 
                  dataKey="value" 
                  name="Portfolio Value" 
                  stroke="#8884d8" 
                  activeDot={{ r: 8 }} 
                />
              </LineChart>
            </ResponsiveContainer>
          </Box>
        </CardContent>
      </Card>
      
      {/* Active Positions */}
      <Typography variant="h5" gutterBottom>
        Active Positions
      </Typography>
      <TableContainer component={Paper} sx={{ mb: 4 }}>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell>Protocol</TableCell>
              <TableCell>Pool</TableCell>
              <TableCell>Assets</TableCell>
              <TableCell align="right">Value</TableCell>
              <TableCell align="right">APY</TableCell>
              <TableCell align="right">Health</TableCell>
              <TableCell>Actions</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {positions.filter(position => position.status === 'Active').map((position) => {
              const protocol = getProtocolById(position.protocol_id);
              const healthStatus = healthStatuses.find(h => h.position_id === position.id);
              
              return (
                <TableRow key={position.id}>
                  <TableCell>{protocol?.name || position.protocol_id}</TableCell>
                  <TableCell>{position.pool_id?.split('-').slice(-3).join('-') || 'N/A'}</TableCell>
                  <TableCell>
                    {position.assets_deposited.map((asset) => (
                      <Chip 
                        key={asset.asset_id}
                        label={`${asset.amount} ${asset.asset_id}`}
                        size="small"
                        sx={{ mr: 0.5, mb: 0.5 }}
                      />
                    ))}
                  </TableCell>
                  <TableCell align="right">{formatCurrency(position.value_usd)}</TableCell>
                  <TableCell align="right">{formatPercentage(position.apy)}</TableCell>
                  <TableCell align="right">
                    {healthStatus ? (
                      <Chip 
                        label={healthStatus.status}
                        color={
                          healthStatus.status === 'Healthy' ? 'success' :
                          healthStatus.status === 'Warning' ? 'warning' : 'error'
                        }
                        size="small"
                      />
                    ) : (
                      <Chip label="N/A" size="small" />
                    )}
                  </TableCell>
                  <TableCell>
                    <Button 
                      size="small" 
                      variant="outlined" 
                      sx={{ mr: 1 }}
                      onClick={() => handleHarvestRewards(position.id)}
                    >
                      Harvest
                    </Button>
                    <Button 
                      size="small" 
                      variant="outlined" 
                      color="secondary"
                      onClick={() => handleClosePosition(position.id)}
                    >
                      Close
                    </Button>
                  </TableCell>
                </TableRow>
              );
            })}
            {positions.filter(position => position.status === 'Active').length === 0 && (
              <TableRow>
                <TableCell colSpan={7} align="center">
                  No active positions
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </TableContainer>
      
      {/* Protocol List */}
      <Typography variant="h5" gutterBottom>
        Available Protocols
      </Typography>
      <Grid container spacing={3}>
        {protocols.map((protocol) => (
          <Grid item xs={12} md={6} lg={4} key={protocol.id}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  {protocol.name}
                </Typography>
                <Chip 
                  label={protocol.protocol_type}
                  size="small"
                  sx={{ mb: 2 }}
                />
                <Typography variant="body2" color="textSecondary" paragraph>
                  {protocol.description}
                </Typography>
                <Divider sx={{ my: 1 }} />
                <Box display="flex" justifyContent="space-between" alignItems="center">
                  <Typography variant="body2">
                    TVL: {formatCurrency(protocol.tvl)}
                  </Typography>
                  <Typography variant="body2">
                    APY: {formatPercentage(protocol.apy_range[0])} - {formatPercentage(protocol.apy_range[1])}
                  </Typography>
                  <Chip 
                    label={protocol.risk_level}
                    color={
                      protocol.risk_level === 'Low' ? 'success' :
                      protocol.risk_level === 'Medium' ? 'info' :
                      protocol.risk_level === 'High' ? 'warning' : 'error'
                    }
                    size="small"
                  />
                </Box>
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>
    </Box>
  );
};

export default DeFiPortfolio;

// frontend/src/components/defi/NewPositionForm.tsx

import React, { useState, useEffect } from 'react';
import { 
  Box, Button, Card, CardContent, CircularProgress, FormControl, 
  Grid, InputLabel, MenuItem, Select, TextField, Typography, Alert
} from '@mui/material';
import { useAuth } from '../../context/AuthContext';
import { DeFiService } from '../../services/DeFiService';
import { formatPercentage } from '../../utils/formatters';

interface Protocol {
  id: string;
  name: string;
  protocol_type: string;
}

interface Pool {
  id: string;
  protocol_id: string;
  name: string;
  assets: string[];
  apy: number;
  risk_level: string;
}

interface Asset {
  asset_id: string;
  amount: number;
}

const NewPositionForm: React.FC = () => {
  const { user } = useAuth();
  const [protocols, setProtocols] = useState<Protocol[]>([]);
  const [pools, setPools] = useState<Pool[]>([]);
  const [selectedProtocol, setSelectedProtocol] = useState<string>('');
  const [selectedPool, setSelectedPool] = useState<string>('');
  const [assets, setAssets] = useState<Asset[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [success, setSuccess] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchProtocols = async () => {
      try {
        const data = await DeFiService.listProtocols();
        setProtocols(data.protocols);
      } catch (err) {
        console.error("Error fetching protocols:", err);
        setError("Failed to load protocols");
      }
    };
    
    fetchProtocols();
  }, []);

  useEffect(() => {
    const fetchPools = async () => {
      if (!selectedProtocol) return;
      
      try {
        const data = await DeFiService.listPools(selectedProtocol);
        setPools(data.pools);
      } catch (err) {
        console.error("Error fetching pools:", err);
        setError("Failed to load pools");
      }
    };
    
    fetchPools();
  }, [selectedProtocol]);

  useEffect(() => {
    if (selectedPool) {
      const pool = pools.find(p => p.id === selectedPool);
      if (pool) {
        setAssets(pool.assets.map(asset => ({ asset_id: asset, amount: 0 })));
      }
    } else {
      setAssets([]);
    }
  }, [selectedPool, pools]);

  const handleProtocolChange = (event: React.ChangeEvent<{ value: unknown }>) => {
    setSelectedProtocol(event.target.value as string);
    setSelectedPool('');
    setAssets([]);
  };

  const handlePoolChange = (event: React.ChangeEvent<{ value: unknown }>) => {
    setSelectedPool(event.target.value as string);
  };

  const handleAssetAmountChange = (index: number, amount: number) => {
    const newAssets = [...assets];
    newAssets[index].amount = amount;
    setAssets(newAssets);
  };

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    
    if (!user || !selectedProtocol || !selectedPool) return;
    
    // Filter out assets with zero amount
    const assetsToDeposit = assets.filter(asset => asset.amount > 0);
    
    if (assetsToDeposit.length === 0) {
      setError("Please enter at least one asset amount");
      return;
    }
    
    setLoading(true);
    setError(null);
    
    try {
      await DeFiService.createPosition(
        user.id,
        selectedProtocol,
        selectedPool,
        assetsToDeposit
      );
      
      setSuccess(true);
      // Reset form
      setSelectedProtocol('');
      setSelectedPool('');
      setAssets([]);
      
      // Clear success message after 3 seconds
      setTimeout(() => {
        setSuccess(false);
      }, 3000);
    } catch (err) {
      console.error("Error creating position:", err);
      setError("Failed to create position. Please check your inputs and try again.");
    } finally {
      setLoading(false);
    }
  };

  const getSelectedPoolDetails = () => {
    return pools.find(p => p.id === selectedPool);
  };

  return (
    <Card>
      <CardContent>
        <Typography variant="h5" gutterBottom>
          Create New Position
        </Typography>
        
        {success && (
          <Alert severity="success" sx={{ mb: 3 }}>
            Position created successfully!
          </Alert>
        )}
        
        {error && (
          <Alert severity="error" sx={{ mb: 3 }}>
            {error}
          </Alert>
        )}
        
        <Box component="form" onSubmit={handleSubmit}>
          <Grid container spacing={3}>
            <Grid item xs={12} md={6}>
              <FormControl fullWidth>
                <InputLabel>Protocol</InputLabel>
                <Select
                  value={selectedProtocol}
                  label="Protocol"
                  onChange={handleProtocolChange}
                  disabled={loading}
                >
                  {protocols.map((protocol) => (
                    <MenuItem key={protocol.id} value={protocol.id}>
                      {protocol.name} ({protocol.protocol_type})
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            </Grid>
            
            <Grid item xs={12} md={6}>
              <FormControl fullWidth disabled={!selectedProtocol || loading}>
                <InputLabel>Pool</InputLabel>
                <Select
                  value={selectedPool}
                  label="Pool"
                  onChange={handlePoolChange}
                >
                  {pools.map((pool) => (
                    <MenuItem key={pool.id} value={pool.id}>
                      {pool.name} - APY: {formatPercentage(pool.apy)}
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            </Grid>
            
            {selectedPool && getSelectedPoolDetails() && (
              <Grid item xs={12}>
                <Typography variant="subtitle1" gutterBottom>
                  Details for {getSelectedPoolDetails()?.name}
                </Typography>
                <Typography variant="body2">
                  APY: {formatPercentage(getSelectedPoolDetails()?.apy || 0)}
                </Typography>
                <Typography variant="body2">
                  Risk Level: {getSelectedPoolDetails()?.risk_level}
                </Typography>
                <Typography variant="body2">
                  Supported Assets: {getSelectedPoolDetails()?.assets.join(', ')}
                </Typography>
              </Grid>
            )}
            
            {assets.length > 0 && (
              <Grid item xs={12}>
                <Typography variant="subtitle1" gutterBottom>
                  Asset Amounts
                </Typography>
                <Grid container spacing={2}>
                  {assets.map((asset, index) => (
                    <Grid item xs={12} md={6} key={asset.asset_id}>
                      <TextField
                        label={`${asset.asset_id} Amount`}
                        type="number"
                        fullWidth
                        value={asset.amount}
                        onChange={(e) => handleAssetAmountChange(index, parseFloat(e.target.value))}
                        InputProps={{
                          inputProps: { min: 0, step: "0.000001" }
                        }}
                        disabled={loading}
                      />
                    </Grid>
                  ))}
                </Grid>
              </Grid>
            )}
            
            <Grid item xs={12}>
              <Button
                type="submit"
                variant="contained"
                color="primary"
                fullWidth
                disabled={loading || !selectedPool || assets.every(a => a.amount <= 0)}
              >
                {loading ? <CircularProgress size={24} /> : "Create Position"}
              </Button>
            </Grid>
          </Grid>
        </Box>
      </CardContent>
    </Card>
  );
};

export default NewPositionForm;

// frontend/src/services/DeFiService.ts

import axios from 'axios';
import { API_URL } from '../config';

const API_ENDPOINT = `${API_URL}/api/v1/defi`;

export const DeFiService = {
  // Protocols
  listProtocols: async () => {
    const response = await axios.get(`${API_ENDPOINT}/protocols`);
    return response.data;
  },
  
  listPools: async (protocolId: string) => {
    const response = await axios.get(`${API_ENDPOINT}/protocols/${protocolId}/pools`);
    return response.data;
  },
  
  // Positions
  listPositions: async (userId: string) => {
    const response = await axios.get(`${API_ENDPOINT}/positions?user_id=${userId}`);
    return response.data;
  },
  
  getPosition: async (userId: string, positionId: string) => {
    const response = await axios.get(`${API_ENDPOINT}/positions/${positionId}?user_id=${userId}`);
    return response.data;
  },
  
  createPosition: async (userId: string, protocolId: string, poolId: string, assets: any[]) => {
    const response = await axios.post(
      `${API_ENDPOINT}/positions?user_id=${userId}`,
      {
        protocol_id: protocolId,
        pool_id: poolId,
        assets: assets
      }
    );
    return response.data;
  },
  
  closePosition: async (userId: string, positionId: string) => {
    const response = await axios.post(
      `${API_ENDPOINT}/positions/${positionId}/close?user_id=${userId}`
    );
    return response.data;
  },
  
  harvestRewards: async (userId: string, positionId: string) => {
    const response = await axios.post(
      `${API_ENDPOINT}/positions/${positionId}/harvest?user_id=${userId}`
    );
    return response.data;
  },
  
  // Portfolio
  getPortfolioValue: async (userId: string) => {
    const response = await axios.get(`${API_ENDPOINT}/portfolio/value?user_id=${userId}`);
    return response.data;
  },
  
  checkPositionsHealth: async (userId: string) => {
    const response = await axios.get(`${API_ENDPOINT}/portfolio/health?user_id=${userId}`);
    return response.data;
  },
};

// frontend/src/utils/formatters.ts

/**
 * Format a number as a currency string
 */
export const formatCurrency = (amount: number): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  }).format(amount);
};

/**
 * Format a number as a percentage string
 */
export const formatPercentage = (value: number): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'percent',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  }).format(value / 100);
};
