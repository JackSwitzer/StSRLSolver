/**
 * Tests for SeedInput component
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '../test-utils';
import { SeedInput } from '../components/SeedInput/SeedInput';
import { useGameStore } from '../store/gameStore';

// Mock the game store
vi.mock('../store/gameStore', () => ({
  useGameStore: vi.fn(),
}));

describe('SeedInput', () => {
  const mockFetchMapData = vi.fn();
  const mockSetAscension = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: null,
      ascension: 20,
      isLoading: false,
      fetchMapData: mockFetchMapData,
      setAscension: mockSetAscension,
    });
  });

  it('renders with default input value', () => {
    render(<SeedInput />);

    const input = screen.getByPlaceholderText('Enter seed...');
    expect(input).toBeInTheDocument();
    expect(input).toHaveValue('A20WIN');
  });

  it('renders ascension selector with options', () => {
    render(<SeedInput />);

    const select = screen.getByRole('combobox');
    expect(select).toBeInTheDocument();
    expect(select).toHaveValue('20');

    // Check ascension options exist
    expect(screen.getByRole('option', { name: 'A0' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'A20' })).toBeInTheDocument();
  });

  it('renders submit button', () => {
    render(<SeedInput />);

    const button = screen.getByRole('button', { name: 'Divine' });
    expect(button).toBeInTheDocument();
    expect(button).not.toBeDisabled();
  });

  it('accepts user input', () => {
    render(<SeedInput />);

    const input = screen.getByPlaceholderText('Enter seed...');
    fireEvent.change(input, { target: { value: 'MYSEED123' } });

    expect(input).toHaveValue('MYSEED123');
  });

  it('triggers search on button click', () => {
    render(<SeedInput />);

    const input = screen.getByPlaceholderText('Enter seed...');
    const button = screen.getByRole('button', { name: 'Divine' });

    fireEvent.change(input, { target: { value: 'TESTSEED' } });
    fireEvent.click(button);

    expect(mockFetchMapData).toHaveBeenCalledWith('TESTSEED', undefined, 20);
  });

  it('triggers search on Enter key press', () => {
    render(<SeedInput />);

    const input = screen.getByPlaceholderText('Enter seed...');

    fireEvent.change(input, { target: { value: 'ENTERSEED' } });
    fireEvent.keyPress(input, { key: 'Enter', code: 'Enter', charCode: 13 });

    expect(mockFetchMapData).toHaveBeenCalledWith('ENTERSEED', undefined, 20);
  });

  it('does not trigger search with empty input', () => {
    render(<SeedInput />);

    const input = screen.getByPlaceholderText('Enter seed...');
    const button = screen.getByRole('button', { name: 'Divine' });

    fireEvent.change(input, { target: { value: '' } });
    fireEvent.click(button);

    expect(mockFetchMapData).not.toHaveBeenCalled();
  });

  it('trims whitespace from seed input', () => {
    render(<SeedInput />);

    const input = screen.getByPlaceholderText('Enter seed...');
    const button = screen.getByRole('button', { name: 'Divine' });

    fireEvent.change(input, { target: { value: '  SPACESEED  ' } });
    fireEvent.click(button);

    expect(mockFetchMapData).toHaveBeenCalledWith('SPACESEED', undefined, 20);
  });

  it('changes ascension and refetches data when seed exists', () => {
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: 'EXISTINGSEED',
      ascension: 20,
      isLoading: false,
      fetchMapData: mockFetchMapData,
      setAscension: mockSetAscension,
    });

    render(<SeedInput />);

    const select = screen.getByRole('combobox');
    fireEvent.change(select, { target: { value: '15' } });

    expect(mockSetAscension).toHaveBeenCalledWith(15);
    expect(mockFetchMapData).toHaveBeenCalledWith('EXISTINGSEED', undefined, 15);
  });

  it('disables controls while loading', () => {
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: null,
      ascension: 20,
      isLoading: true,
      fetchMapData: mockFetchMapData,
      setAscension: mockSetAscension,
    });

    render(<SeedInput />);

    expect(screen.getByPlaceholderText('Enter seed...')).toBeDisabled();
    expect(screen.getByRole('combobox')).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Loading...' })).toBeDisabled();
  });

  it('shows loading text on button when loading', () => {
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: null,
      ascension: 20,
      isLoading: true,
      fetchMapData: mockFetchMapData,
      setAscension: mockSetAscension,
    });

    render(<SeedInput />);

    expect(screen.getByRole('button', { name: 'Loading...' })).toBeInTheDocument();
  });
});
